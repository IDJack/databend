
select '===Explain===';
drop table if exists t1;
drop table if exists t2;
create table t1(a int, b int);
create table t2(a int, b int);
explain select t1.a from t1 where a > 0;
explain select * from t1, t2 where (t1.a = t2.a and t1.a > 3) or (t1.a = t2.a and t2.a > 5 and t1.a > 1);
explain select * from t1, t2 where (t1.a = t2.a and t1.a > 3) or (t1.a = t2.a);
drop table t1;
drop table t2;
set enable_planner_v2 = 0;
