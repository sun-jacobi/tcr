# A toy C compiler in Rust

### Reference
+ https://www.sigbus.info/compilerbook By Rui Ueyama
+ https://doc.rust-lang.org/stable/nightly-rustc/rustc_lexer/index.html (About the implementation of lexer in Rust)
+ https://www2.cs.arizona.edu/classes/cs453/fall14/DOCS/cminusminusspec.html (A BNF syntax for a subset of C)


### Example
```
int foo(a) 
{
  if (a == 1) 
    return 1; 
  else 
    return a + foo(a - 1);
} 
int main()
{
  return foo(1);
}
```

