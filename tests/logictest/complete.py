#!/usr/bin/env python3
# -*- coding: UTF-8 -*-
import os

from argparse import ArgumentParser
import mysql.connector

from logictest import is_empty_line
from config import mysql_config
from http_connector import format_result

target_dir = "./"

def parse_sql_file(source_file):
    sqls = list()
    f = open(source_file, encoding='UTF-8')
    sql_content = ""
    for line in f.readlines():
        if is_empty_line(line):
            continue

        if line.startswith("--"):  # pass comment
            continue

        # multi line sql
        sql_content = sql_content + line.rstrip()
        if ';' not in line:
            continue

        statement = sql_content.strip()
        sqls.append(statement)
        sql_content = ""

    f.close()
    return sqls

def parse_logictest_file(source_file):
    parsing_statement = False
    sqls = list()
    f = open(source_file, encoding='UTF-8')
    sql_content = "" 
        
    for line in f.readlines():
        if is_empty_line(line):
            if parsing_statement:
                parsing_statement = False
                statement = sql_content.strip()
                if ";" not in statement:
                    statement = statement + ";"
                sqls.append(statement)
                sql_content = ""
            continue

        if line.startswith("----") and parsing_statement:
            parsing_statement = False
            statement = sql_content.strip()
            if ";" not in statement:
                statement = statement + ";"
            sqls.append(statement)
            sql_content = ""

        if line.startswith("--") or line.startswith("#"):  # pass comment
            continue

        if line.startswith("statement") or line.startswith("query"):
            parsing_statement = True
            continue

        if parsing_statement:
            sql_content = sql_content + line.rstrip()
            if ';' not in line:
                continue
            parsing_statement = False
            statement = sql_content.strip()
            sqls.append(statement)
            sql_content = ""

    f.close()
    return sqls

def get_sql_from_file(source_file):
    if ".sql" in os.path.basename(source_file):
        return parse_sql_file(source_file)
    else:
        return parse_logictest_file(source_file)


def gen_suite_from_sql(sqls, dest_file):
    out = open(f"{dest_file}", mode="w+", encoding='UTF-8')
    statements = list()
    connection = mysql.connector.connect(**mysql_config)
    cursor = connection.cursor(buffered=True)
    for sql in sqls:
        # use mysql connector
        try:
            cursor.execute(sql)
        except mysql.connector.Error as err:
            statements.append(f"statement error {err.errno}\n{sql}\n\n")
            continue

        try:
            r = cursor.fetchall()
            results = []
            options = ""
            for (ri, row) in enumerate(r):
                rows = []
                for (i, v) in enumerate(row):
                    if len(options) <= i:
                        if isinstance(v, int):
                            options += "I"
                        elif isinstance(v, float):
                            options += "F"
                        else:
                            options += "T"

                    if isinstance(v, type(None)):
                        rows.append("NULL")
                        continue
                    rows.append(str(v))
                results.append(rows)
            statements.append(f"statement query {options}\n{sql}\n\n----\n{format_result(results)}\n")
        except mysql.connector.Error as err:
            statements.append(f"statement ok\n{sql}\n\n")

    out.writelines(statements)
    out.flush()
    out.close()


def run(args):
    if not os.path.isfile(args.source_file):
        print(f"{args.source_file} is not a file")
        return
    print(f"Source file: {args.source_file}")
    print(f"Dest file: {args.dest_file}")

    mysql_config['user'] = args.mysql_user
    mysql_config['host'] = args.mysql_host
    mysql_config['port'] = args.mysql_port
    mysql_config['passwd'] = args.mysql_passwd
    mysql_config['database'] = args.mysql_database

    print(f"Mysql config: {mysql_config}")
    sqls = get_sql_from_file(args.source_file)
    if args.show_sql:
        for sql in sqls:
            print(sql)

    gen_suite_from_sql(sqls, args.dest_file)


if __name__ == '__main__':
    parser = ArgumentParser(description='databend sqllogictest auto-complete tools(from *.sql files or logictest files)')
    parser.add_argument('--source-file',
                        help='Path to suites source file')

    parser.add_argument('--dest-file',
                        default="./auto",
                        help='Path to logictest auto-complete file')

    parser.add_argument('--show-sql',
                        action='store_true',
                        default=False,
                        help='Show sql from source file')

    parser.add_argument('--mysql-user',
                        default="root",
                        help='Mysql user')

    parser.add_argument('--mysql-host',
                        default="127.0.0.1",
                        help='Mysql host')

    parser.add_argument('--mysql-port',
                        default="3307",
                        help='Mysql port')

    parser.add_argument('--mysql-passwd',
                        default="root",
                        help='Mysql password')

    parser.add_argument('--mysql-database',
                        default="default",
                        help='Mysql default database')
    
    args = parser.parse_args()
    run(args)
