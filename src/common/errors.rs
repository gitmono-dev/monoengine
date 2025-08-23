//! MonoEngine 错误处理模块
//! 
//! 该模块定义了 MonoEngine 项目的统一错误处理机制，
//! 提供了错误类型定义、错误转换和错误处理的相关功能。
//!

use thiserror::Error;
use anyhow::anyhow;

/// MonoEngine 的主要错误类型
/// 
/// 该结构体封装了应用程序中可能出现的各种错误，
/// 包含错误信息和对应的错误代码
#[derive(Error, Debug)]
pub struct MonoError {
    /// 可选的错误信息，使用 anyhow::Error 提供丰富的错误上下文
    pub error: Option<anyhow::Error>,
    /// 错误代码，用于程序退出时的状态码
    pub code: i32,
}

impl MonoError {
    /// 创建一个新的 MonoError 实例
    /// 
    /// # 参数
    /// 
    /// * `error` - anyhow::Error 类型的错误信息
    /// * `code` - 错误代码
    /// 
    /// # 返回值
    /// 
    /// 返回新创建的 MonoError 实例
    pub fn new(error: anyhow::Error, code: i32) -> MonoError {
        MonoError {
            error: Some(error),
            code,
        }
    }

    /// 打印错误信息
    ///
    /// 之前该方法通过 panic! 终止程序，这会在仅需要输出错误时导致
    /// 整个应用崩溃。改为输出到标准错误，调用者可自行决定后续处理。
    pub fn print(&self) {
        if let Some(err) = &self.error {
            eprintln!("{}:{}", self.code, err);
        }
    }

    /// 创建未知子命令错误
    /// 
    /// # 参数
    /// 
    /// * `cmd` - 未知的子命令名称
    /// 
    /// # 返回值
    /// 
    /// 返回包含未知子命令错误信息的 MonoError
    pub fn _unknown_subcommand(cmd: impl AsRef<str>) -> MonoError {
        MonoError {
            error: anyhow!("Unknown subcommand: {}", cmd.as_ref()).into(),
            code: 1,
        }
    }

    /// 创建带有自定义消息的错误
    /// 
    /// # 参数
    /// 
    /// * `msg` - 自定义错误消息
    /// 
    /// # 返回值
    /// 
    /// 返回包含自定义消息的 MonoError
    pub fn _with_message(msg: impl AsRef<str>) -> MonoError {
        MonoError {
            error: anyhow!("Error Message: {}", msg.as_ref()).into(),
            code: 0,
        }
    }
}

/// 为 MonoError 实现 Display trait
/// 
/// 允许 MonoError 被格式化为字符串输出
impl std::fmt::Display for MonoError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.error.as_ref().unwrap())
    }
}

/// 从 anyhow::Error 转换为 MonoError
/// 
/// 默认错误代码为 101
impl From<anyhow::Error> for MonoError {
    fn from(err: anyhow::Error) -> MonoError {
        MonoError::new(err, 101)
    }
}

/// 从 clap::Error 转换为 MonoError
/// 
/// 根据 clap 错误的类型设置相应的错误代码
impl From<clap::Error> for MonoError {
    fn from(err: clap::Error) -> MonoError {
        let code = i32::from(err.use_stderr());
        MonoError::new(err.into(), code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    /// 测试 MonoError::new 方法
    #[test]
    fn test_mono_error_new() {
        let error = anyhow!("测试错误");
        let mono_error = MonoError::new(error, 42);
        
        assert!(mono_error.error.is_some());
        assert_eq!(mono_error.code, 42);
        assert!(mono_error.to_string().contains("测试错误"));
    }

    /// 测试 MonoError::unknown_subcommand 方法
    #[test]
    fn test_unknown_subcommand() {
        let mono_error = MonoError::_unknown_subcommand("invalid_cmd");
        
        assert!(mono_error.error.is_some());
        assert_eq!(mono_error.code, 1);
        assert!(mono_error.to_string().contains("Unknown subcommand: invalid_cmd"));
    }

    /// 测试 MonoError::with_message 方法
    #[test]
    fn test_with_message() {
        let mono_error = MonoError::_with_message("自定义错误消息");
        
        assert!(mono_error.error.is_some());
        assert_eq!(mono_error.code, 0);
        assert!(mono_error.to_string().contains("Error Message: 自定义错误消息"));
    }

    /// 测试 Display trait 实现
    #[test]
    fn test_display() {
        let error = anyhow!("显示测试");
        let mono_error = MonoError::new(error, 1);
        let display_string = format!("{}", mono_error);
        
        assert!(display_string.contains("显示测试"));
    }

    /// 测试从 anyhow::Error 的转换
    #[test]
    fn test_from_anyhow_error() {
        let anyhow_error = anyhow!("anyhow 错误");
        let mono_error: MonoError = anyhow_error.into();
        
        assert!(mono_error.error.is_some());
        assert_eq!(mono_error.code, 101);
        assert!(mono_error.to_string().contains("anyhow 错误"));
    }

    /// 测试从 clap::Error 的转换
    #[test]
    fn test_from_clap_error() {
        use clap::{Arg, Command};
        
        // 创建一个简单的 clap 命令来生成错误
        let cmd = Command::new("test")
            .arg(Arg::new("required")
                .required(true)
                .help("必需参数"));
        
        // 尝试解析空参数列表，这会产生错误
        let clap_error = cmd.try_get_matches_from(["test"]).unwrap_err();
        let mono_error: MonoError = clap_error.into();
        
        assert!(mono_error.error.is_some());
        // clap 错误的代码应该是基于 use_stderr() 的结果
        assert!(mono_error.code == 0 || mono_error.code == 1);
    }

    /// 测试错误链
    #[test]
    fn test_error_chain() {
        let root_cause = anyhow!("根本原因");
        let wrapped_error = root_cause.context("包装错误");
        let mono_error = MonoError::new(wrapped_error, 500);
        
        assert!(mono_error.error.is_some());
        assert_eq!(mono_error.code, 500);
        let error_string = mono_error.to_string();
        assert!(error_string.contains("包装错误"));
    }

    /// 测试错误代码的不同值
    #[test]
    fn test_different_error_codes() {
        let error1 = MonoError::_with_message("错误1");
        let error2 = MonoError::_unknown_subcommand("cmd");
        let error3 = MonoError::from(anyhow!("错误3"));

        assert_eq!(error1.code, 0);
        assert_eq!(error2.code, 1);
        assert_eq!(error3.code, 101);
    }

    /// 确保 `print` 方法不会触发 panic
    #[test]
    fn test_print_does_not_panic() {
        let error = MonoError::_with_message("打印测试");
        let result = std::panic::catch_unwind(|| {
            error.print();
        });
        assert!(result.is_ok());
    }
}
