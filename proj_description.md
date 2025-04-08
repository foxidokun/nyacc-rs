### Итерация 1: Грамматика
Сгенерирована автоматически из [описания](src/grammar.lalrpop)

Все ноды покрыты тестами на подобии данного из [кода обработки for](src/ast/statement/for_st.rs):
```rust
check_ast!(
    StatementParser,
    "for (a = 3; a < 100; a = a * 2) {a = 3; a = 7;}",
    ast_node!(
        For,
        ast_node!(
            Assignment,
            Variable::new("a".into(), vec![]),
            ast_node!(Int, 3)
        ),
        ast_node!(
            Compare,
            ast_node!(Variable, "a".into(), vec![]),
            Comparator::LT,
            ast_node!(Int, 100)
        ),
        ast_node!(
            Assignment,
            Variable::new("a".into(), vec![]),
            ast_node!(
                Arithmetic,
                ast_node!(Variable, "a".into(), vec![]),
                OpType::Mul,
                ast_node!(Int, 2)
            )
        ),
        vec![
            ast_node!(
                Assignment,
                Variable::new("a".into(), vec![]),
                ast_node!(Int, 3)
            ),
            ast_node!(
                Assignment,
                Variable::new("a".into(), vec![]),
                ast_node!(Int, 7)
            )
        ]
    )
);
```

### Итерация 2: Visitor
Интерфейс визитора состоит из функций вида `fn visit_function_call(&mut self, node: &FunctionCall)` где `node` это нода AST дерева.

Данное апи генерируется в полуавтоматическом режиме в [visitor.rs](src/visitor.rs):
```rust
trait Visitor {
    acceptor_func!(Assignment);
    acceptor_func!(For);
    acceptor_func!(FunctionCall);
    // ...
}
```

Далее каждая нода AST с помощью реализованной [магии](lib/proc/src/lib.rs) `#[derive(Acceptor)]` реализует метод интерфейса Acceptor в таком ключе:
```rust
// Имплементация для вышеупомянутого FunctionCall
impl Acceptor for FunctionCall {
    fn accept(&self, visitor: &mut dyn Visitor) -> anyhow::Result<()> {
        visitor.visit_functioncall(self)?;
        Ok(())
    }
}
```

С помощью апи визиторов написаны:
— таргет AST в [debug.rs](src/ast/debug.rs)
— первый проход, находящий все обявления функций и типов в [definitions.rs](src/codegen/definitions.rs) (строки `impl Visitor for ProgramDefinitions`)


### Итерация 3: IR (+ codegen)
Реализовано через интерфейсы Statement и Expression, которые предоставляют методы `codegen`. Для Statement `codegen` ничего не возвращает (не считая ошибок), для Expression возвращает обертку над llvm::Value, содержающую информацию о типе выражения. Благодаря этому можно опускать тип в `let`, не беспокоиться о кастах при вызове функций и арифметических операциях

Кодген также покрыт тестами с помощью JIT, например в [simple.rs](src/codegen/tests/simple.rs) реализована проверка FFI вызова:

```rust
#[unsafe(no_mangle)]
pub extern "C" fn test_id(a: i32) -> i32 {
    a
}

#[test]
fn ffi_cal() {
    check_codegen!(
        "
        fn test_id(a: i32) -> i32;

        fn mul(a: i32, b: i32) -> i32 { return test_id(a) * b; }
        ",
        [extern test_id],
        [mul as fn(i32, i32) -> i32],
        [assert mul(1, 2) == 2]
    )
}
```

Если честно, я горд за тестирование в этом компиляторе

### Итерация 4: Таблица символов
To be described later...

Но она существует, в любой момент можно узнать какого типа переменная, поддерживается shadowing переменных (через стек информации по каждому блоку видимости). Соответственно тип функции так же можно узнать. Для переменных можно так же получить их llvm::Value