use git2::{self, Repository};
use thiserror::Error;
use std::path::PathBuf;
use anyhow::{anyhow, Error as AnyhowError};
use std::process::{Command, Stdio};
use std::string::FromUtf8Error;
use fil_actor_init::{InstallParams, InstallReturn};
use fil_actors_runtime::INIT_ACTOR_ADDR;
use forest_key_management::KeyInfo;
use mpool::{mpool_push, MpoolError};
use rpc::RpcEndpoint;
use fvm_shared::{
    econ::TokenAmount,
    address::Address,
};
use fvm_ipld_encoding_3::{
    RawBytes,
};
use forest_json::{
    cid::CidJson,
};

#[derive(Debug, Error)]
pub enum ActorError {
    #[error("git call error")]
    GitCallError(#[from] git2::Error),
    #[error("io call error")]
    IOCallError(#[from] std::io::Error),
    #[error("common error")]
    CommonError(#[from] AnyhowError),
    #[error("parse json error")]
    ParseJsonError(#[from] serde_json::Error),
    #[error("parse utf8 error")]
    ParseUtf8Error(#[from] FromUtf8Error),
    #[error("mpool call error")]
    MpoolCallError(#[from] MpoolError),
}

pub fn clone_actor(repo_url: &str, target_path: PathBuf) -> Result<(), ActorError> {
    let _ = Repository::clone(repo_url, target_path)?;
    Ok(())
}

pub fn compile_actor(target_path: PathBuf) -> Result<PathBuf, ActorError> {
    Command::new("cargo")
        .current_dir(target_path.clone())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .arg("build")
        .arg("--release")
        .output()?;

    let output = Command::new("cargo")
        .current_dir(target_path.clone())
        .arg("read-manifest")
        .output()?;

    if !output.status.success() {
        return Err(ActorError::CommonError(anyhow!("fail read manifest")));
    }

    let manifest = String::from_utf8(output.stdout)?;
    let value = serde_json::from_str::<serde_json::Value>(&manifest)?;
    let name = value.get("name").ok_or(ActorError::CommonError(anyhow!("invalid name")))?;

    let wasm_path = target_path.join("target/wasm32-unknown-unknown/release");
    let mut wasm_path = wasm_path.join(name.as_str().ok_or(ActorError::CommonError(anyhow!("invalid name")))?);
    wasm_path.set_extension("wasm");

    Ok(wasm_path)
}

pub async fn install_actor(
    rpc: RpcEndpoint,
    target_path: PathBuf,
    from: Address,
    from_key_info: KeyInfo,
) -> Result<(CidJson, bool), ActorError> {
    let code = std::fs::read(target_path)?;
    let code = RawBytes::from(code);
    let params = InstallParams {
        code: code,
    };

    let ret = mpool_push::<_, InstallReturn>(
        rpc,
        from,
        from_key_info,
        INIT_ACTOR_ADDR,
        4,
        TokenAmount::from_atto(0),
        vec![params],
    ).await?;

    Ok((CidJson(ret.code_cid), ret.installed))
}

pub fn create_actor() {
    println!("{}", " Try create actor");
}

pub fn take_owner() {
    println!("{}", " Try take owner");
}
