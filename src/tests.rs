use rocket::serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Test {
    pub name: String,
    pub elements: Vec<Question>
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
                options[n_opt].clone().into()
            }
        }
    }
}

pub fn make_tipi_test() -> Test {
    use QuestionContent::*;
    let likert7 = vec![
        "Disagree strongly".into(),
        "Disagree moderately".into(),
        "Disagree a little".into(),
        "Neither agree nor disagree".into(),
        "Agree a little".into(),
        "Agree moderately".into(),
        "Agree strongly".into()
    ];
    Test {
        name: "Ten Item Personality Inventory".into(),
        elements: vec![
            Question { id: "ep".into(), content: McQuestion { text: "Extraverted, enthusiastic".into(), options: likert7.clone() } },
            Question { id: "am".into(), content: McQuestion { text: "Critical, quarrelsome".into(), options: likert7.clone() } },
            Question { id: "cp".into(), content: McQuestion { text: "Dependable, self-disciplined".into(), options: likert7.clone() } },
            Question { id: "np".into(), content: McQuestion { text: "Anxious, easily upset".into(), options: likert7.clone() } },
            Question { id: "op".into(), content: McQuestion { text: "Open to new experiences, complex".into(), options: likert7.clone() } },
            Question { id: "em".into(), content: McQuestion { text: "Reserved, quiet".into(), options: likert7.clone() } },
            Question { id: "ap".into(), content: McQuestion { text: "Sympathetic, warm".into(), options: likert7.clone() } },
            Question { id: "cm".into(), content: McQuestion { text: "Disorganized, careless".into(), options: likert7.clone() } },
            Question { id: "nm".into(), content: McQuestion { text: "Calm, emotionally stable".into(), options: likert7.clone() } },
            Question { id: "om".into(), content: McQuestion { text: "Conventional, uncreative".into(), options: likert7.clone() } },
        ]
    }
}
