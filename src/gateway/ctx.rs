use std::{collections::HashMap, fmt::Debug};

use async_trait::async_trait;
use futures::future::join_all;

use crate::Result;

pub type Id = usize;

#[derive(Debug, Clone)]
pub struct Ctx {
    pub app_id: Id,
    pub endpoint_id: Id,
}

#[async_trait]
pub trait ConfigToContext {
    type Context;

    async fn into_context(self) -> Result<Self::Context>;
}

#[macro_export]
macro_rules! config_into_context {
    ($type:ty) => {
        #[async_trait::async_trait]
        impl $crate::ConfigToContext for $type {
            type Context = $type;

            async fn into_context(self) -> $crate::Result<Self::Context> {
                Ok(self)
            }
        }
    };
}

config_into_context!(i8);
config_into_context!(i16);
config_into_context!(i32);
config_into_context!(i64);
config_into_context!(u8);
config_into_context!(u16);
config_into_context!(u32);
config_into_context!(u64);
config_into_context!(f32);
config_into_context!(f64);
config_into_context!(bool);
config_into_context!(());

#[async_trait]
impl<I: Send> ConfigToContext for Vec<I> {
    type Context = Box<[I]>;

    async fn into_context(self) -> Result<Self::Context> {
        Ok(self.into_boxed_slice())
    }
}

#[async_trait]
impl<I: ConfigToContext + Send> ConfigToContext for Option<I> {
    type Context = Option<I::Context>;

    async fn into_context(self) -> Result<Self::Context> {
        match self {
            Some(i) => Ok(Some(i.into_context().await?)),
            None => Ok(None),
        }
    }
}

#[async_trait]
impl ConfigToContext for String {
    type Context = Box<str>;

    async fn into_context(self) -> Result<Self::Context> {
        Ok(self.into_boxed_str())
    }
}

#[derive(Debug)]
pub struct MiddlewareConfig<Global, Endpoints>(HashMap<String, AppConfig<Global, Endpoints>>);

impl<
        GlobalCtx,
        GlobalConf: ConfigToContext<Context = GlobalCtx>,
        EndpointsCtx,
        EndpointsConf: ConfigToContext<Context = EndpointsCtx>,
    > MiddlewareConfig<GlobalConf, EndpointsConf>
{
    pub fn new(data: HashMap<String, AppConfig<GlobalConf, EndpointsConf>>) -> Self {
        Self(data)
    }

    pub async fn into_context(
        mut self,
        ids: &[String],
        routers: &HashMap<String, Vec<String>>,
    ) -> Result<MiddlewareCtx<GlobalCtx, EndpointsCtx>> {
        let futures = ids
            .iter()
            .map(|id| (routers.get(id), self.0.remove(id)))
            .map(|(router, config)| async move {
                match (router, config) {
                    (Some(route_ids), Some(config)) => match config.into_context(route_ids).await {
                        Ok(context) => Ok(Some(context)),
                        Err(e) => Err(e),
                    },
                    _ => Ok(None),
                }
            });
        let context = join_all(futures)
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;
        Ok(MiddlewareCtx(context.into_boxed_slice()))
    }
}

impl<
        GlobalCtx,
        GlobalConf: ConfigToContext<Context = GlobalCtx>,
        EndpointsCtx,
        EndpointsConf: ConfigToContext<Context = EndpointsCtx>,
    > From<HashMap<String, AppConfig<GlobalConf, EndpointsConf>>>
    for MiddlewareConfig<GlobalConf, EndpointsConf>
{
    fn from(value: HashMap<String, AppConfig<GlobalConf, EndpointsConf>>) -> Self {
        Self::new(value)
    }
}

impl<
        GlobalCtx,
        GlobalConf: ConfigToContext<Context = GlobalCtx>,
        EndpointsCtx,
        EndpointsConf: ConfigToContext<Context = EndpointsCtx>,
    > From<(GlobalConf, HashMap<String, EndpointsConf>)> for AppConfig<GlobalConf, EndpointsConf>
{
    fn from(value: (GlobalConf, HashMap<String, EndpointsConf>)) -> Self {
        Self(value.0, value.1)
    }
}

#[derive(Debug)]
pub struct AppConfig<Global, Endpoints>(Global, HashMap<String, Endpoints>);

impl<
        GlobalCtx,
        GlobalConf: ConfigToContext<Context = GlobalCtx>,
        EndpointsCtx,
        EndpointsConf: ConfigToContext<Context = EndpointsCtx>,
    > AppConfig<GlobalConf, EndpointsConf>
{
    async fn into_context(mut self, ids: &[String]) -> Result<AppCtx<GlobalCtx, EndpointsCtx>> {
        let futures = ids.iter().map(|id| self.1.remove(id)).map(|config| async {
            match config {
                Some(config) => match config.into_context().await {
                    Ok(context) => Ok(Some(context)),
                    Err(e) => Err(e),
                },
                None => Ok(None),
            }
        });
        let context = join_all(futures)
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;
        Ok(AppCtx(
            self.0.into_context().await?,
            context.into_boxed_slice(),
        ))
    }
}

#[derive(Debug)]
pub struct MiddlewareCtx<Global, Endpoints>(Box<[Option<AppCtx<Global, Endpoints>>]>);

unsafe impl<Global, Endpoints> Send for MiddlewareCtx<Global, Endpoints> {}
unsafe impl<Global, Endpoints> Sync for MiddlewareCtx<Global, Endpoints> {}

impl<Global, Endpoints> MiddlewareCtx<Global, Endpoints> {
    pub fn get(&self, id: Id) -> Option<&AppCtx<Global, Endpoints>> {
        self.0.get(id).and_then(Option::as_ref)
    }
}

#[derive(Debug)]
pub struct AppCtx<Global, Endpoints>(Global, Box<[Option<Endpoints>]>);

impl<Global, Endpoints> AppCtx<Global, Endpoints> {
    pub fn global(&self) -> &Global {
        &self.0
    }
    pub fn get(&self, id: Id) -> Option<&Endpoints> {
        self.1.get(id).and_then(Option::as_ref)
    }
}
