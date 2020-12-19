import optuna
import subprocess
from joblib import Parallel, delayed
import json

d = None

def func(trial):
    d2 = dict()
    for (k, v) in d.items():
        if isinstance(v, list):
            d2[k] = trial.suggest_uniform(k, v[0], v[1])
        else:
            d2[k] = v

    with open("./data/params.json", 'w') as f:
        json.dump(d2, f, indent=4)

    proc = subprocess.run("./target/release/procon31-comp", shell=True, stdout=subprocess.PIPE, text=True);
    res = proc.stdout
    lis = res.split('\n')
    lis = list(filter(lambda x: len(x) >= 1, lis))
    print(lis)
    x = lis[-1]
    return int(x)

def run():
    study = optuna.load_study(study_name="opt_study", storage="sqlite:///./opt_study.db")
    study.optimize(func, n_trials=5)

def main():
    with open("./data/params_default.json") as f:
        global d
        d = json.load(f)

    study = optuna.create_study(study_name="opt_study", direction="maximize", storage="sqlite:///./opt_study.db", load_if_exists="True")
    print(study.best_value, study.best_params)

    Parallel(n_jobs=12)([delayed(run)() for _ in range(12)])

if __name__ == '__main__':
    main()
