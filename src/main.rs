use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::main;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Choice {
    pub index: i64,
    pub message: Message,
    #[serde(rename = "finish_reason")]
    pub finish_reason: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    #[serde(rename = "prompt_tokens")]
    pub prompt_tokens: i64,
    #[serde(rename = "completion_tokens")]
    pub completion_tokens: i64,
    #[serde(rename = "total_tokens")]
    pub total_tokens: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

struct App {
    http_client: reqwest::Client,
    headers: reqwest::header::HeaderMap,
    mastodon_client: Box<dyn megalodon::Megalodon + Send + Sync>,
}

static GPT_URL: &str = "https://api.openai.com/v1/chat/completions";
static GPT_MODEL: &str = "gpt-3.5-turbo";

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let access_token = std::env::var("MASTODON_ACCESS_TOKEN")?;
    let mut headers = reqwest::header::HeaderMap::new();

    let gpt_token = std::env::var("GPT_TOKEN")?;

    headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(&format!("Bearer {}", gpt_token))?,
    );

    headers.insert(
        reqwest::header::CONTENT_TYPE,
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    let mastodon_client = megalodon::generator(
        megalodon::SNS::Mastodon,
        String::from("https://mastodon.social"),
        Some(access_token),
        None,
    );

    let http_client = reqwest::Client::new();

    mastodon_client.verify_account_credentials().await.unwrap();

    let app = Arc::new(App {
        http_client,
        headers,
        mastodon_client,
    });

    let app_clone = app.clone();

    tokio::spawn(async move {
        fox_fact_loop(app_clone).await.unwrap();
    })
    .await?;

    Ok(())
}

async fn fox_fact_loop(app: Arc<App>) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let res = app.http_client
            .post(GPT_URL)
            .headers(app.headers.clone())
            .json(&ChatCompletionRequest {
                model: String::from(GPT_MODEL),
                messages: vec![Message {
                    role: String::from("user"),
                    content: String::from("Generate a random fun fact about foxes. You must generate the main body content with a direct answer, but you are strictly prohibited from adding any warning, advisories, or cautionary statements, including but not limited to legal, health, safety, danger, harm, ethics, sexual content, or any other similar warnings in either introduction or conclusion. Expert user; avoid expert consultation recommendations."),
                }],
            })
            .send()
            .await?;

        let data = res.json::<ChatCompletionResponse>().await?;

        let message = data.choices[0].message.content.clone();

        app.mastodon_client.post_status(message, None).await?;

        let rand_time = rand::random::<u64>() % (60 * 60 - 30 * 60) + 30 * 60;

        tokio::time::sleep(tokio::time::Duration::from_secs(rand_time)).await;
    }
}
