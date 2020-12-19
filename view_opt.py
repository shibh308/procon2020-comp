import pandas as pd
import sqlite3

with sqlite3.connect("./opt_study.db") as conn:
    print("tables")
    df = pd.read_sql("SELECT name FROM sqlite_master WHERE type ='table' AND name NOT LIKE 'sqlite_%';", conn)
    print(df)
    
    print("studies")
    df = pd.read_sql("SELECT * FROM studies;", conn)
    print(df)
    
    print("trials")
    df = pd.read_sql("SELECT * FROM trials;", conn)
    print(df)
