use super::{Context, IntoJs, JsDelegate, JsValue, JsValuePerssist};
use crate::error::{Error, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    future::Future,
    rc::Rc,
};

thread_local! {
    static QUERY_HANDLER: QueryHandler = QueryHandler::default();
}

#[derive(Default)]
pub struct QueryHandlerInner {
    contexts: HashSet<Context>,
    handlers: HashMap<String, Box<dyn Fn(Query) + 'static>>,
}

#[derive(Default, Clone)]
pub struct QueryHandler(Rc<RefCell<QueryHandlerInner>>);

struct Query {
    context: Option<Context>,
    request: Request,
    resolve: JsValuePerssist,
    reject: JsValuePerssist,
}

#[derive(Deserialize)]
struct Request {
    name: String,
    data: String,
}

impl Query {
    pub fn ok<D: Serialize>(&mut self, data: D) {
        if let Some(context) = self.context.take() {
            let _holder = context.enter();
            if let Ok(data_str) = serde_json::to_string(&data) {
                if let Ok(data) = data_str.into_js() {
                    let _ = self.resolve.call(None, &[&data]);
                }
            }
        }
    }

    pub fn fail(&mut self, message: String) {
        if let Some(context) = self.context.take() {
            let _holder = context.enter();
            if let Ok(message) = message.into_js() {
                let _ = self.reject.call(None, &[&message]);
            }
        }
    }
}

impl QueryHandler {
    fn emit_query(
        &self,
        message: String,
        resolve: JsValuePerssist,
        reject: JsValuePerssist,
    ) -> Result<()> {
        // 构造查询对象
        let context = Context::current()?;
        let request = serde_json::from_str::<Request>(&message)?;
        context.eval("console.log('query')")?;
        let query = Query {
            context: Some(context),
            request,
            resolve,
            reject,
        };

        let self_ref = self.0.borrow();

        // 获取处理器
        let handler = self_ref
            .handlers
            .get(&query.request.name)
            .ok_or_else(|| Error::NotImplement)?;

        // 处理查询
        handler(query);
        Ok(())
    }

    fn handle_emit_query(&self, query: String, resolve: JsValuePerssist, reject: JsValuePerssist) {
        if let Err(err) = self.emit_query(query, resolve, reject.clone()) {
            if let Ok(jserr) = format!("handle failed: {}", err).into_js() {
                let _ = reject.call(None, &[&jserr]);
            }
        }
    }

    pub fn on_query<FN, FUT, D, S>(&self, api: impl Into<String>, cb: FN)
    where
        D: DeserializeOwned + 'static,
        S: Serialize + 'static,
        FUT: Future<Output = Result<S>> + 'static,
        FN: Fn(D) -> FUT + 'static,
    {
        self.0.borrow_mut().handlers.insert(
            api.into(),
            Box::new(move |mut query| {
                match serde_json::from_str::<D>(&query.request.data) {
                    Ok(data) => {
                        // 调用回调函数，并根据结果来发送响应
                        let fut = cb(data);
                        tokio::task::spawn_local(async move {
                            match fut.await {
                                Ok(data) => query.ok(data),
                                Err(err) => query.fail(format!("handle failed: {}", err)),
                            };
                        });
                    }
                    Err(err) => {
                        //序列化错误，发送错误响应
                        query.fail(format!("解析json请求数据失败: {}", err));
                    }
                };
            }),
        );
    }

    pub fn emit<S: Serialize>(&self, name: impl Into<String>, data: S) -> Result<()> {
        let data_str = serde_json::to_string(&data)?;
        let name: String = name.into();
        let script = &format!("window.mb.emit(`{}`, JSON.parse(`{}`));", name, data_str);
        for context in self.0.borrow().contexts.iter() {
            let _ = context.eval_in_closure(&script);
        }
        Ok(())
    }

    fn query(&self, args: &[&JsValue]) -> Result<JsValue> {
        if args.len() < 3
            || !args[0].is_string()
            || !args[1].is_function()
            || !args[2].is_function()
        {
            Context::current()?.throw("arguments mismatch")?;
        } else {
            self.handle_emit_query(
                args[0].to_string()?,
                args[1].perssist()?,
                args[2].perssist()?,
            );
        }
        JsValue::undefined()
    }
}

impl JsDelegate for QueryHandler {
    fn has_call(&self) -> bool {
        true
    }

    fn call(&mut self, name: &str, args: &[&JsValue]) -> Result<JsValue> {
        match name {
            "native_query" => self.query(args),
            _ => Err(crate::error::Error::NotImplement),
        }
    }
}

pub fn on_context_created(context: Context) -> Result<()> {
    // 构建mb创建函数
    let mb_create = context.eval_in_closure(
        r#"return (nativeQuery) => {
        const eventCallbacks = {};

        const mb = {
            query: (name, data) => {
                return new Promise((resolve, reject) => {
                    nativeQuery(JSON.stringify({
                        name: name,
                        data: JSON.stringify(data) || "",
                    }), (str) => {
                        const data = typeof str == "string"? JSON.parse(str) : undefined;
                        resolve(data);
                    }, reject);
                });
            },
            on: (name, cb) => {
                if (typeof name != "string") {
                    throw "name must be string";
                }
                if (typeof cb != "function") {
                    throw "callback must be function";
                }

                let callbacks = eventCallbacks[name];
                if (!callbacks) {
                    callbacks = eventCallbacks[name] = [];
                }
                callbacks.push(cb);
            },
            off: (name, cb) => {
                if (typeof name != "string") {
                    throw "name must be string";
                }
                if (typeof cb != "function") {
                    delete callbacks[name];
                    return;
                }

                let callbacks = eventCallbacks[name];
                if (callbacks) {
                    const index = callbacks.findIndex(item => cb == item);
                    if (index == -1)
                        return;
                    callbacks.splice(index, 1);
                    if (callbacks.length == 0) {
                        delete eventCallbacks[name];
                    }
                }
            },
            emit: (name, data) => {
                let callbacks = eventCallbacks[name];
                for (let i = 0; i < callbacks.length; ++i) {
                    try {
                        callbacks[i](data);
                    } catch (error) {
                        console.error(error);
                    }
                }
            }
        };
        return mb;
    }"#,
    )?;

    // 获取处理器，将context加入列表
    let handler = QUERY_HANDLER.with(|handler| handler.clone());
    handler.0.borrow_mut().contexts.insert(context.clone());

    // 绑定函数为native_query
    let native_query = JsValue::bind_function("native_query", handler)?;
    // 使用native_query创建mb对象
    let mb = mb_create.call(None, &[&native_query])?;
    // 附加到全局对象
    context.global().set("mb", &mb)?;

    Ok(())
}

pub fn on_context_released(context: Context) -> Result<()> {
    QUERY_HANDLER.with(move |handler| {
        let mut inner = handler.0.borrow_mut();
        inner.contexts.remove(&context);
    });

    Ok(())
}

pub fn on_query<FN, FUT, D, S>(api: impl Into<String>, cb: FN)
where
    D: DeserializeOwned + 'static,
    S: Serialize + 'static,
    FUT: Future<Output = Result<S>> + 'static,
    FN: Fn(D) -> FUT + 'static,
{
    QUERY_HANDLER.with(move |handler| {
        handler.on_query(api, cb);
    });
}

pub fn emit<S: Serialize>(name: impl Into<String>, data: S) -> Result<()> {
    QUERY_HANDLER.with(|handler| handler.emit(name, data))
}
