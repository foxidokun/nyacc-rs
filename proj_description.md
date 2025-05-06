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

### Итерация 5: Проверка типов

В силу "си подобности" методов нет. Проверка типов производится во время кодгена при попытках скастовать. Так, оператор присваивания в случае явного указанного типа попытается скастовать (функция `cast`):
```rust
let mut expr = self.expr.codegen(cxt)?;
if let Some(typename) = &self.tp {
    let ty = cxt.definitions.get_type(typename);
    if ty.is_none() {
        anyhow::bail!("Unknown type {} in let statement", typename);
    }
    let ty = ty.unwrap();
    expr.value = cast(cxt, &expr.ty, &ty, expr.value)?;
    expr.ty = ty;
}
```

Кастовать можно из любого типа в него же самого, либо Float <-> Int -> Bool, все остальные попытки приведут к ошибке компиляции

### Итерация 6: Готовый компилятор

В силу желания вместо бинарника код автоматически jit исполняется. Верхнеуровневый обработчиком jit является фцнкция 
```rust
pub fn jit_target(prog: &Program) -> anyhow::Result<()> {
    let mut cxt = CodegenContext::prepare(prog)?;
    prog.codegen(&mut cxt)?;

    // [вызов кодгена]
    let ee = JitEngine::from_codegen_cxt(cxt);

    // [регистрация стандартных функций]
    nyastd::register_functions(|name: &'static str, addr| {
        /* Currently here we ignore Err's, cause probably they caused by unimported functions
         * but as TODO we should add std func definitions into func. But it requires proc_macro magic
         * for parsing function types
         */
        let _ = ee.add_func_mapping(name, addr);
    });

    // [Запускает оптимизатор]
    run_optimizer(ee.module);

    // [Вытаскивает указатель на main и запускает ее]
    let func_ptr = ee.get_func_addr("main")?;
    let func_ptr: fn() -> () = unsafe { std::mem::transmute(func_ptr) };

    func_ptr();

    Ok(())
}
```
которая вызывает кодген, строит jit execute'ор, оптимизирует код и запускает его. 

Построение jit:
```rust
pub fn from_codegen_cxt(mut cxt: CodegenContext) -> Self {
        let ee = unsafe {
            LLVMLinkInMCJIT();
            LLVM_InitializeNativeTarget();
            LLVM_InitializeNativeAsmPrinter();

            // Build an execution engine.
            {
                let mut ee = std::mem::MaybeUninit::uninit();
                let mut err = std::mem::zeroed();

                // This moves ownership of the module into the execution engine.
                if LLVMCreateExecutionEngineForModule(ee.as_mut_ptr(), cxt.module, &mut err) != 0 {
                    // In case of error, we must avoid using the uninitialized ExecutionEngineRef.
                    assert!(!err.is_null());
                    panic!(
                        "Failed to create execution engine: {:?}",
                        CStr::from_ptr(err)
                    );
                }

                ee.assume_init()
            }
        };
        /* ... snip ... */
    }
```

Оптимизатор с target triple:
```rust
fn run_optimizer(module: *mut LLVMModule) {
    unsafe { LLVM_InitializeNativeTarget() };

    let triple = unsafe { LLVMGetDefaultTargetTriple() };
    assert!(!triple.is_null());

    let target = unsafe {
        let mut err = null_mut();
        let mut target = std::mem::MaybeUninit::uninit();
        let res = LLVMGetTargetFromTriple(triple, target.as_mut_ptr(), &mut err);
        if res != 0 {
            // In case of error, we must avoid using the uninitialized ExecutionEngineRef.
            assert!(!err.is_null());
            panic!(
                "Failed to create execution engine: {:?}",
                CStr::from_ptr(err)
            );
        }

        target.assume_init()
    };
    assert!(!target.is_null());

    let cpu = unsafe { LLVMGetHostCPUName() };
    assert!(!cpu.is_null());

    let features = unsafe { LLVMGetHostCPUFeatures() };
    assert!(!features.is_null());

    let machine = unsafe {
        LLVMCreateTargetMachine(
            target,
            triple,
            cpu,
            features,
            LLVMCodeGenLevelDefault, // O2
            LLVMRelocStatic,
            LLVMCodeModelDefault,
        )
    };
    assert!(!machine.is_null());

    let options = unsafe { LLVMCreatePassBuilderOptions() };
    assert!(!options.is_null());

    unsafe { LLVMRunPasses(module, c"default<O2>".as_ptr(), machine, options) };

    /* Cleanup */
    unsafe {
        LLVMDisposeTargetMachine(machine);
        LLVMDisposePassBuilderOptions(options);
        LLVMDisposeMessage(triple);
        LLVMDisposeMessage(cpu);
        LLVMDisposeMessage(features);
    }
}
```

Регистрация стандартных функций через callback + LLVMAddGlobalMapping, чтобы можно было вызывать rust функции из jit кода

```rust
// nyastd/src/lib.rs
pub fn register_functions<T>(mut callback: T)
where T: FnMut(&'static str, *mut ())
{
    macro_rules! export_symbol {
        ($symbol:ident) => {
            callback(stringify!($symbol), $symbol as *mut ());
        };
    }

    export_symbol!(print_int);
    export_symbol!(read_int);
}


// src/codegen.rs
pub fn jit_target(prog: &Program) -> anyhow::Result<()> {
    /* ... */
    nyastd::register_functions(|name: &'static str, addr| {
        let _ = ee.add_func_mapping(name, addr);
    });
    /* ... */
}

// src/codegen/context.rs
pub fn add_func_mapping(&self, name: &str, obj: *mut ()) -> anyhow::Result<()> {
        let func_name = CString::new(name).unwrap();
        let func = unsafe { LLVMGetNamedFunction(self.module, func_name.as_ptr() as *const _) };
        if func.is_null() {
            anyhow::bail!("Function {} wasn;t imported", name);
        }

        unsafe {
            LLVMAddGlobalMapping(self.ee, func, obj as *mut c_void);
        }

        Ok(())
    }
```