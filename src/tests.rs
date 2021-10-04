use std::collections::HashMap;
use std::convert::Infallible;
use rocket::http::uri::fmt::{FromUriParam, Part};
use rocket::request::{FromParam};
use rocket::serde::{Serialize, Serializer};
use serde_json::{json, Value};
use lazy_static::lazy_static;
use crate::util::contains;

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
    pub condition: Condition,
    pub elements: Vec<Question>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub enum Condition {
    Always,
    Question { id: String, value: Value }
}
impl Condition {
    pub fn eval(&self, resp: HashMap<String, Value>) -> bool {
        match &self {
            Condition::Always => true,
            Condition::Question { id, value } => {
                let comp = &resp[id];
                contains(value, comp)
            }
        }
    }
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
    Paragraph { header: String, paragraph: String },
    McQuestion { text: String, options: Vec<String> },
    McQuestionVert { text: String, options: Vec<String>, other: bool },
    CheckboxQuestion { text: String },
    TextAreaQuestion { header: String, paragraph: String },
}
impl Question {
    pub fn convert(&self, resp: &HashMap<String, String>) -> Option<Value> {
        use QuestionContent::*;
        match &self.content {
            Paragraph { .. } => None,
            McQuestion { options, .. } => {
                if let Some(answer) = resp.get(&self.id) {
                    let n_opt: usize = answer.parse().unwrap();
                    Some(json!({"ord": n_opt, "nom": options[n_opt].clone()}))
                }
                else { None }
            }
            McQuestionVert { options, .. } => {
                if let Some(answer) = resp.get(&self.id) {
                    let n_opt: usize = answer.parse().unwrap();
                    if n_opt == options.len() {
                        Some(json!({"nom": "Other", "answer": resp.get(&format!("{:?}.other", &self.id))}))
                    }
                    else {
                        Some(json!({"ord": n_opt, "nom": options[n_opt].clone()}))
                    }
                }
                else { None }
            }
            CheckboxQuestion { text } => {
                if let Some(answer) = resp.get(&self.id) {
                    Some(json!({"text": text, "checked": answer == "on"}))
                }
                else {
                    Some(json!({"declined": text, "checked": false}))
                }
            }
            TextAreaQuestion { .. } => {
                Some(json!({"answer": resp[&self.id].clone()}))
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
        use QuestionContent::*;
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
    let mut test = {
        use QuestionContent::*;
        Test {
            id: "tipi".into(),
            name: "Ten Item Personality Inventory".into(),
            pages: vec![
                TestPage {
                    condition: Condition::Always,
                    elements: {
                        test_items.insert(0, Question {
                            id: "".into(),
                            content: Paragraph {
                                header: "TIPI Personality Test".into(),
                                paragraph: "Here are a number of personality traits that may or may not \
                                apply to you. Please select an option for each statement to indicate \
                                the extent to which you agree or disagree with that statement. You \
                                should rate the extent to which the pair of traits applies to you, \
                                even if one characteristic applies more strongly than the other.".into()
                            }
                        }); test_items
                    }
                },
                TestPage {
                    condition: Condition::Always,
                    elements: vec![
                        Question {
                            id: "".into(),
                            content: Paragraph {
                                header: "Meta".into(),
                                paragraph: "Before you get your feedback, there's just a few extra \
                                questions that I would like to know your answer to. These questions \
                                don't affect your test result, but they are good to know on my end \
                                so I know what to make of your response.".into()
                            }
                        },
                        Question {
                            id: "accurate".into(),
                            content: CheckboxQuestion {
                                text: "My response is accurate to the best of my ability".into(),
                            }
                        },
                        Question {
                            id: "repeat".into(),
                            content: CheckboxQuestion {
                                text: "I remember having taken this test before on this website".into(),
                            }
                        },
                        Question {
                            id: "additional".into(),
                            content: CheckboxQuestion {
                                text: "I would be open to answering a few extra demographic questions \
                                   to contribute to research (you will be presented with another \
                                   page on the test if you check this)".into(),
                            }
                        },
                    ]
                },
                TestPage {
                    condition: Condition::Question { id: "additional".into(), value: json!({ "checked": true }) },
                    elements: vec![
                        Question {
                            id: "".into(),
                            content: Paragraph {
                                header: "Demographics".into(),
                                paragraph: "Thank you for volunteering to answering demographic questions; \
                                it helps me understand who my visitors are and how the norms for the \
                                test differs between groups. Please answer the questions below.".into()
                            }
                        },
                        Question {
                            id: "gender".into(),
                            content: McQuestionVert {
                                text: "Gender".into(),
                                options: vec!["Male".into(), "Female".into()],
                                other: true
                            },
                        },
                    ],
                },
                TestPage {
                    condition: Condition::Always,
                    elements: vec![
                        Question {
                            id: "".into(),
                            content: Paragraph {
                                header: "Consent & End".into(),
                                paragraph: "Thank you for using my website to take the test. Before \
                                continuing, you can optionally consent to allowing your previous \
                                responses to be published in a dataset in the future. Your response \
                                will be anonymous, except for what you've chosen to share in the survey.".into(),
                            }
                        },
                        Question {
                            id: "consent".into(),
                            content: CheckboxQuestion {
                                text: "My response may anonymously be entered into public datasets to \
                                   support future research".into(),
                            }
                        },
                        Question {
                            id: "comments".into(),
                            content: TextAreaQuestion {
                                header: "Comments".into(),
                                paragraph: "Do you have any comments before submitting your response? \
                                For privacy reasons, these comments will be kept private even if you \
                                consent to having your data shared in the above question.".into(),
                            }
                        },
                    ],
                }],
            feedback: vec![],
        }
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
