# rie

RIE is rust interactive editor. 

REPL-like expirience based on interactive source code editing. 

```
>> :2 * 2
= 4

>> let x = 2
= 
>> :x * 2
= 4

>> let f = |x : i32| x * x
= 
>> :f(4)
= 16

>> %
0 fn main() {
1     let x = 2;
2     let f = |x : i32| x * x;
3 }
>> %d 2
0 fn main() {
1     let x = 2;
2 }
>> 
```
