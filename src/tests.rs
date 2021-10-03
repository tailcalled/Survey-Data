use std::collections::HashMap;
use std::convert::Infallible;
use rocket::http::uri::fmt::{FromUriParam, Part};
use rocket::request::{FromParam};
use rocket::serde::{Serialize, Serializer};
use serde_json::{json, Value};
use lazy_static::lazy_static;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Test {
    pub id: String,
    pub name: String,
    pub pages: Vec<TestPage>,
    pub feedback: Vec<FeedbackItem>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct TestPage {
    pub elements: Vec<Question>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Question {
    pub id: String,
    pub content: QuestionContent
}
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub enum QuestionContent {
    McQuestion { text: String, options: Vec<String> }
}
impl QuestionContent {
    pub fn convert(&self, resp: &str) -> Value {
        use QuestionContent::*;
        match self {
            McQuestion { options, .. } => {
                let n_opt: usize = resp.parse().unwrap();
                json!({"ord": n_opt, "nom": options[n_opt].clone()})
            }
        }
    }
}
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub enum FeedbackItem {
    Title { text: String },
    Paragraph { text: String },
    Bar { score: f64, min: f64, max: f64 },
    Score { eval: Scorer },
}
pub struct Scorer(Box<dyn Send + Sync + Fn(&Value) -> FeedbackItem>);

impl Serialize for Scorer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        "N/A".serialize(serializer)
    }
}

impl FeedbackItem {
    pub(crate) fn score(&self, value: &Value) -> FeedbackItem {
        use FeedbackItem::*;
        match self {
            Score { eval: Scorer(eval) } => eval(value),
            Title { text } => Title { text: text.clone() },
            Paragraph { text } => Paragraph { text: text.clone() },
            Bar { score, min, max } => Bar { score: *score, min: *min, max: *max },
        }
    }
}

pub fn make_tipi_test() -> Test {
    use QuestionContent::*;
    use FeedbackItem::*;
    let likert7 = vec![
        "Disagree strongly".into(),
        "Disagree moderately".into(),
        "Disagree a little".into(),
        "Neither agree nor disagree".into(),
        "Agree a little".into(),
        "Agree moderately".into(),
        "Agree strongly".into()
    ];
    let mut test_items = vec![];
    let mut add_item = |id: &str, label: &str| {
        test_items.push(Question {
            id: id.into(),
            content: McQuestion {
                text: label.into(),
                options: likert7.clone(),
            }
        })
    };
    add_item("ep", "Extraverted, enthusiastic");
    add_item("am", "Critical, quarrelsome");
    add_item("cp", "Dependable, self-disciplined");
    add_item("np", "Anxious, easily upset");
    add_item("op", "Open to new experiences, complex");
    add_item("em", "Reserved, quiet");
    add_item("ap", "Sympathetic, warm");
    add_item("cm", "Disorganized, careless");
    add_item("nm", "Calm, emotionally stable");
    add_item("om", "Conventional, uncreative");
    let mut test = Test {
        id: "tipi".into(),
        name: "Ten Item Personality Inventory".into(),
        pages: vec![TestPage { elements: test_items }],
        feedback: vec![],
    };
    let mut add_score = |label: &str, pos: &'static str, neg: &'static str, descr: &str| {
        test.feedback.push(Title { text: label.into() });
        test.feedback.push(Paragraph { text: descr.into() });
        test.feedback.push(Score { eval: Scorer(Box::new(move |resp: &Value| {
            let num_pos: f64 = resp[pos]["ord"].as_f64().unwrap();
            let num_neg: f64 = resp[neg]["ord"].as_f64().unwrap();
            Bar { score: 1.0 + (num_pos + (6.0 - num_neg)) / 2.0, min: 1.0, max: 7.0 }
        }))})
    };
    add_score("Extraversion", "ep", "em",
              "Extraversion is characterized by warmth, gregariousness, assertiveness, \
              activity, excitement seeking, and positive emotions.");
    add_score("Agreeableness", "ap", "am",
              "Agreeableness is characterized by trust, straightforwardness, altruism, \
              compliance, modesty and tender-mindedness.");
    add_score("Conscientiousness", "cp", "cm",
              "Conscientiousness is characterized by competence, orderliness, dutifulness, \
              achievement-striving, self-discipline and deliberation.");
    add_score("Neuroticism", "np", "nm",
              "Neuroticism is characterized by anxiety, anger, depression, \
              self-consciousness, impulsiveness and vulnerability.");
    add_score("Openness", "np", "nm",
              "Openness is characterized by fantasy, aesthetic interests, depth of feelings, \
              adventurousness, intellectual interests, and liberalism.");
    test
}

pub struct Tests(HashMap<String, Test>);

pub fn make_tests() -> Tests {
    let mut tests = HashMap::new();
    tests.insert("tipi".into(), make_tipi_test());
    Tests(tests)
}

lazy_static! {
    static ref TESTS: Tests = make_tests();
}

impl<'a> FromParam<'a> for &Test {
    type Error = Infallible;

    fn from_param(param: &'a str) -> Result<Self, Infallible> {
        Ok(TESTS.0.get(param).unwrap())
    }
}

impl<'r, P: Part> FromUriParam<P, &'r Test> for &Test {
    type Target = &'r str;
    fn from_uri_param(param: &'r Test) -> &'r str {
        &param.id
    }
}
