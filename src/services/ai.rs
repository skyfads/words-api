use once_cell::sync::OnceCell;
use openai_api_rust::chat::*;
use openai_api_rust::*;

pub struct AIService {
    client: OpenAI,
    default_model: String,
    default_temperature: f32,
    default_max_tokens: i32,
}

impl AIService {
    pub fn new_from_env() -> Self {
        let auth = Auth::from_env().expect("OPENAI_API_KEY not set");
        let client = OpenAI::new(auth, "https://api.openai.com/v1/");

        AIService {
            client,
            default_model: "gpt-4.1-mini".to_string(),
            default_temperature: 0.0,
            default_max_tokens: 300,
        }
    }

    pub fn chat(&self, prompt: &str) -> Result<String, openai_api_rust::Error> {
        let body = ChatBody {
            model: self.default_model.clone(),
            max_tokens: Some(self.default_max_tokens),
            temperature: Some(self.default_temperature),
            top_p: Some(1.0),
            n: Some(1),
            stream: Some(false),
            stop: None,
            presence_penalty: Some(0.0),
            frequency_penalty: Some(0.0),
            logit_bias: None,
            user: None,
            messages: vec![Message {
                role: Role::User,
                content: prompt.to_string(),
            }],
        };

        let resp = self.client.chat_completion_create(&body)?;
        let message = &resp.choices[0].message.as_ref().unwrap();
        Ok(message.content.clone())
    }
}

pub static AI_SERVICE: OnceCell<AIService> = OnceCell::new();

pub fn init_ai_service() -> &'static AIService {
    AI_SERVICE.get_or_init(|| AIService::new_from_env())
}
