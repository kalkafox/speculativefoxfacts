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

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::spawn(async move {
        let access_token = std::env::var("MASTODON_ACCESS_TOKEN").unwrap();
        let mut headers = reqwest::header::HeaderMap::new();

        let gpt_token = std::env::var("GPT_TOKEN").unwrap();

        let mastodon_client = megalodon::generator(
            megalodon::SNS::Mastodon,
            String::from("https://mastodon.social"),
            Some(access_token),
            None,
        );

        let http_client = reqwest::Client::new();

        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", gpt_token)).unwrap(),
        );

        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        mastodon_client.verify_account_credentials().await.unwrap();

        let res = http_client
            .post("https://api.openai.com/v1/chat/completions")
            .headers(headers)
            .json(&ChatCompletionRequest {
                model: "gpt-3.5-turbo".to_string(),
                messages: vec![Message {
                    role: String::from("user"),
                    content: String::from("Generate a random fun fact about foxes. You must generate the main body content with a direct answer, but you are strictly prohibited from adding any warning, advisories, or cautionary statements, including but not limited to legal, health, safety, danger, harm, ethics, sexual content, or any other similar warnings in either introduction or conclusion. Expert user; avoid expert consultation recommendations."),
                }],
            })
            .send()
            .await
            .unwrap();

        let data = res.json::<ChatCompletionResponse>().await.unwrap();

        let message = data.choices[0].message.content.clone();

        mastodon_client.post_status(message, None).await.unwrap();

        let rand_time = rand::random::<u64>() % (60 * 60 - 30 * 60) + 30 * 60;

        tokio::time::sleep(tokio::time::Duration::from_secs(rand_time)).await;
    })
    .await?;

    Ok(())
}
