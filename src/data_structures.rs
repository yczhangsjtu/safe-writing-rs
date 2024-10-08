use crate::{
    cipher::{decrypt, encrypt},
    encode::base64_encode,
    error::Error,
};

pub(crate) const IMAGE_SEP: u8 = 0x88;

#[derive(Debug, Clone)]
pub struct Passage {
    id: usize,
    title: String,
    content: String,
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

#[derive(Default, Debug, Clone)]
pub struct PlainText {
    next_id: usize,
    content: Vec<Passage>,
    images: Vec<Vec<u8>>,
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
        self.content[index].content = content;
    }

    pub fn set_title(&mut self, index: usize, title: String) {
        self.content[index].title = title;
    }

    pub fn passages(&self) -> &Vec<Passage> {
        &self.content
    }

    pub fn remove_passage(&mut self, index: usize) -> Passage {
        self.content.remove(index)
    }

    pub fn append_plaintext(&mut self, plaintext: &PlainText) {
        self.content.extend(plaintext.content.clone());
    }

    pub fn bounded_index(&self, index: usize) -> usize {
        if self.content.len() == 0 {
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
        encrypt(password, self)
    }

    pub fn decrypt(password: &str, ciphertext: &str) -> Result<Self, Error> {
        let ciphertext = ciphertext.split("\n").collect::<Vec<_>>();
        if ciphertext.len() < 3 {
            return Err(Error::DecryptionFail);
        }
        decrypt(password, ciphertext[0], ciphertext[1], ciphertext[2])
    }
}
