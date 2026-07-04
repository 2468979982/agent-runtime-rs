// examples/basic_usage.rs
// 基本用法示例：创建 Agent 运行时并发送消息
// 运行方式：cargo run --example basic_usage

use agent_runtime_rs::create_agent_runtime;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("🚀 Initializing Agent Runtime...");

    // 创建 Agent 运行时
    let runtime = create_agent_runtime(
        "config/agent-config.json",
        "config/tools-config.json",
        "config/prompt-config.json",
    )
    .await?;

    println!("✅ Agent Runtime initialized successfully");

    // 示例 1：简单对话
    println!("\n📝 Example 1: Simple chat");
    let response = runtime
        .chat(
            Some("example-session-1"),
            "Hello! What can you do?".to_string(),
        )
        .await?;
    println!("Agent: {}", response.response);

    // 示例 2：使用工具
    println!("\n📝 Example 2: Using tools");
    let response = runtime
        .chat(
            Some("example-session-1"),
            "Calculate 123 * 456".to_string(),
        )
        .await?;
    println!("Agent: {}", response.response);

    // 示例 3：多轮对话（会话保持上下文）
    println!("\n📝 Example 3: Multi-turn conversation");
    let response = runtime
        .chat(
            Some("example-session-1"),
            "What did I just ask you to calculate?".to_string(),
        )
        .await?;
    println!("Agent: {}", response.response);

    // 示例 4：新会话（无上下文）
    println!("\n📝 Example 4: New session (no context)");
    let response = runtime
        .chat(
            Some("example-session-2"),
            "What did I just ask you to calculate?".to_string(),
        )
        .await?;
    println!("Agent: {}", response.response);

    println!("\n✅ All examples completed successfully!");

    Ok(())
}
