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
Во время кодгена существует [CodegenContext](https://git.foxido.dev/foxido/nyacc-rs/-/blob/master/src/codegen/context.rs?ref_type=heads#L102), который внутри себя хранит `definitions: ProgramDefinitions`, которые в свою очередь являются ничем иным, как
```rust
type FuncType = (Vec<Rc<Type>>, Rc<Type>);

pub struct ProgramDefinitions {
    /// typename => typedata
    pub types: HashMap<String, Rc<Type>>,
    /// func_name => func_info
    functions: HashMap<String, Rc<FuncType>>,
}
```

> Rc<T> является ничем иным как местным shared_ptr<T> (Reference Counter)

Объявленные переменные и функции можно узнать из `cxt.vislayers: VisibilityContext`, являющегося ничем иным как
```rust
struct TypedValue {
    value: *mut LLVMValue,
    ty: Rc<Type>,
}

struct VisibilityContext {
    layers: Vec<HashMap<String, TypedValue>>,
    cur_func: Option<(*mut LLVMValue, Rc<Type>)>,
}
```
где layers является стеком объявленных переменных, чтобы поддерживать скоупы. 

В свою очередь `Type` есть ни что иное как enum (в растовом понимании, что-то близкое к std::variant):
```rust
pub enum Type {
    Void(),
    Float(FloatType),
    Int(IntType),
    Custom(CustomType),
}
```
Подробнее в [definitions.rs](src/codegen/definitions.rs)

Переобъявление переменной просто перекрывает предыдущее [test_redefinition](src/codegen/tests/simple.rs), использование неопределененной переменной приводит к ошибке компиляции [test_unknown_var](src/codegen/tests/compilation_errors.rs),
shadowing поддерживается, причем с использованием предыдущей при объявлении следующей [test_shadowing/test_self_shadowing](src/codegen/tests/simple.rs)