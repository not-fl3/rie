# rie

RIE is REPL-like interactive code editor.
``` 
# cargo install rie
# rie

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
>> {{
>>> fn foo() {
>>>   println!("Hello world");
>>> }
>>> }}
= 
>> :foo()
= Hello world
()
```
