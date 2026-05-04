use crate::{
    encode::{base64_decode, base64_encode},
    error::Error,
};

pub(crate) const IMAGE_SEP: u8 = 0x88;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Passage {
    pub id: usize,
    pub title: String,
    pub content: String,
}

impl Passage {
    pub fn new(id: usize, title: String, content: String) -> Self {
        Self { id, title, content }
    }

    pub fn title(&self) -> &String {
        &self.title
    }

    pub fn content(&self) -> &String {
        &self.content
    }

    pub fn encode(&self) -> String {
        let title = base64_encode(self.title.as_bytes());
        let content = base64_encode(self.content.as_bytes());
        title + "-" + &content
    }
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlainText {
    pub next_id: usize,
    pub content: Vec<Passage>,
    #[serde(skip)]
    pub images: Vec<Vec<u8>>,
}

impl PlainText {
    pub fn new(next_id: usize, content: Vec<Passage>, images: Vec<Vec<u8>>) -> Self {
        Self {
            next_id,
            content,
            images,
        }
    }

    pub fn empty() -> Self {
        Self::new(0, vec![], vec![])
    }

    pub fn from_passages(content: Vec<Passage>) -> Self {
        Self::new(0, content, vec![])
    }

    pub fn from_passages_images(content: Vec<Passage>, images: Vec<Vec<u8>>) -> Self {
        Self::new(0, content, images)
    }

    pub fn num_passages(&self) -> usize {
        self.content.len()
    }

    pub fn num_images(&self) -> usize {
        self.images.len()
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn get_first_passage_text(&self) -> Option<String> {
        self.content.get(0).map(|p| p.content.clone())
    }

    pub fn title_of_passage(&self, index: usize) -> Option<String> {
        self.content.get(index).map(|p| p.title.clone())
    }

    pub fn content_of_passage(&self, index: usize) -> Option<&String> {
        self.content.get(index).map(|p| &p.content)
    }

    pub fn content_of_passage_mut(&mut self, index: usize) -> Option<&mut String> {
        self.content.get_mut(index).map(|p| &mut p.content)
    }

    pub fn images(&self) -> &Vec<Vec<u8>> {
        &self.images
    }

    pub fn images_mut(&mut self) -> &mut Vec<Vec<u8>> {
        &mut self.images
    }

    pub fn id_of_passage(&self, index: usize) -> Option<usize> {
        self.content.get(index).map(|p| p.id)
    }

    pub fn set_content(&mut self, index: usize, content: String) {
        if index < self.content.len() {
            self.content[index].content = content;
        }
    }

    pub fn set_title(&mut self, index: usize, title: String) {
        if index < self.content.len() {
            self.content[index].title = title;
        }
    }

    pub fn passages(&self) -> &Vec<Passage> {
        &self.content
    }

    pub fn remove_passage(&mut self, index: usize) -> Passage {
        self.content.remove(index)
    }

    pub fn append_plaintext(&mut self, plaintext: &PlainText) {
        self.content.extend(plaintext.content.clone());
        self.images.extend(plaintext.images.clone());
    }

    pub fn bounded_index(&self, index: usize) -> usize {
        if self.content.is_empty() {
            0
        } else if index < self.content.len() {
            index
        } else {
            self.content.len() - 1
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let passages_data = (self
            .content
            .iter()
            .map(|p| p.encode())
            .collect::<Vec<_>>()
            .join("|")
            + ":FontSize=24")
            .as_bytes()
            .to_vec();
        if self.images.is_empty() {
            passages_data
        } else {
            vec![
                passages_data,
                vec![IMAGE_SEP],
                (self.images.len() as u32).to_le_bytes().to_vec(),
                self.images
                    .iter()
                    .map(|image| {
                        let mut image_data = (image.len() as u32).to_le_bytes().to_vec();
                        image_data.extend_from_slice(image);
                        image_data
                    })
                    .collect::<Vec<_>>()
                    .concat(),
            ]
            .concat()
        }
    }

    pub fn insert_new_passage(&mut self, index: usize, title: String) {
        self.content.insert(
            index,
            Passage {
                id: self.next_id,
                title,
                content: "".to_string(),
            },
        );
        self.next_id += 1;
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        self.content.swap(a, b);
    }

    pub fn encrypt(&self, password: &str) -> String {
        crate::cipher::encrypt(password, self)
    }

    pub fn decrypt(password: &str, ciphertext: &str) -> Result<Self, Error> {
        let ciphertext = ciphertext.split("\n").collect::<Vec<_>>();
        if ciphertext.len() < 3 {
            return Err(Error::DecryptionFail);
        }
        crate::cipher::decrypt(password, ciphertext[0], ciphertext[1], ciphertext[2])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passage_encode_decode() {
        let passage = Passage::new(0, "Test Title".to_string(), "Test Content".to_string());
        let encoded = passage.encode();
        assert!(encoded.contains("-"));
    }

    #[test]
    fn test_plaintext_empty() {
        let empty = PlainText::empty();
        assert!(empty.is_empty());
        assert_eq!(empty.num_passages(), 0);
        assert_eq!(empty.num_images(), 0);
    }

    #[test]
    fn test_plaintext_from_passages() {
        let passages = vec![
            Passage::new(0, "Title 1".to_string(), "Content 1".to_string()),
            Passage::new(1, "Title 2".to_string(), "Content 2".to_string()),
        ];
        let plaintext = PlainText::from_passages(passages);
        assert_eq!(plaintext.num_passages(), 2);
        assert_eq!(plaintext.title_of_passage(0), Some("Title 1".to_string()));
        assert_eq!(plaintext.content_of_passage(1), Some(&"Content 2".to_string()));
    }

    #[test]
    fn test_plaintext_insert_passage() {
        let mut plaintext = PlainText::empty();
        plaintext.insert_new_passage(0, "New Title".to_string());
        assert_eq!(plaintext.num_passages(), 1);
        assert_eq!(plaintext.title_of_passage(0), Some("New Title".to_string()));
        assert_eq!(plaintext.content_of_passage(0), Some(&"".to_string()));
    }

    #[test]
    fn test_plaintext_swap() {
        let mut plaintext = PlainText::from_passages(vec![
            Passage::new(0, "A".to_string(), "Content A".to_string()),
            Passage::new(1, "B".to_string(), "Content B".to_string()),
        ]);
        plaintext.swap(0, 1);
        assert_eq!(plaintext.title_of_passage(0), Some("B".to_string()));
        assert_eq!(plaintext.title_of_passage(1), Some("A".to_string()));
    }

    #[test]
    fn test_plaintext_remove() {
        let mut plaintext = PlainText::from_passages(vec![
            Passage::new(0, "A".to_string(), "Content A".to_string()),
            Passage::new(1, "B".to_string(), "Content B".to_string()),
        ]);
        let removed = plaintext.remove_passage(0);
        assert_eq!(removed.title, "A");
        assert_eq!(plaintext.num_passages(), 1);
        assert_eq!(plaintext.title_of_passage(0), Some("B".to_string()));
    }

    #[test]
    fn test_plaintext_encode_empty() {
        let empty = PlainText::empty();
        let encoded = empty.encode();
        let decoded = String::from_utf8(encoded).unwrap();
        assert!(decoded.contains(":FontSize=24"));
    }

    #[test]
    fn test_plaintext_set_content() {
        let mut plaintext = PlainText::from_passages(vec![
            Passage::new(0, "Title".to_string(), "Old Content".to_string()),
        ]);
        plaintext.set_content(0, "New Content".to_string());
        assert_eq!(plaintext.content_of_passage(0), Some(&"New Content".to_string()));
    }

    #[test]
    fn test_plaintext_set_title() {
        let mut plaintext = PlainText::from_passages(vec![
            Passage::new(0, "Old Title".to_string(), "Content".to_string()),
        ]);
        plaintext.set_title(0, "New Title".to_string());
        assert_eq!(plaintext.title_of_passage(0), Some("New Title".to_string()));
    }

    #[test]
    fn test_plaintext_append() {
        let mut plaintext1 = PlainText::from_passages(vec![
            Passage::new(0, "A".to_string(), "Content A".to_string()),
        ]);
        let plaintext2 = PlainText::from_passages(vec![
            Passage::new(1, "B".to_string(), "Content B".to_string()),
        ]);
        plaintext1.append_plaintext(&plaintext2);
        assert_eq!(plaintext1.num_passages(), 2);
    }
}