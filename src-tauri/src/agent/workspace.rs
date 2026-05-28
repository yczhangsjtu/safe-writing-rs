use serde::{Deserialize, Serialize};

/// A character in the story
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: usize,
    pub name: String,
    pub description: String,
    pub aliases: Vec<String>,
    pub traits: Vec<String>,
    pub notes: String,
}

/// A relationship between two characters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: usize,
    pub character_a_id: usize,
    pub character_b_id: usize,
    pub relationship_type: String,
    pub description: String,
}

/// A key-value entry for storing arbitrary data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValueEntry {
    pub key: String,
    pub value: String,
    pub category: String,
    pub notes: String,
}

/// The workspace containing characters, relationships, and key-value store
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Workspace {
    pub next_character_id: usize,
    pub characters: Vec<Character>,
    pub next_relationship_id: usize,
    pub relationships: Vec<Relationship>,
    pub key_value_store: Vec<KeyValueEntry>,
}

impl Workspace {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new character
    pub fn add_character(&mut self, name: String, description: String) -> Character {
        let character = Character {
            id: self.next_character_id,
            name,
            description,
            aliases: Vec::new(),
            traits: Vec::new(),
            notes: String::new(),
        };
        self.next_character_id += 1;
        self.characters.push(character.clone());
        character
    }

    /// Update a character
    pub fn update_character(&mut self, id: usize, name: Option<String>, description: Option<String>, aliases: Option<Vec<String>>, traits: Option<Vec<String>>, notes: Option<String>) -> Result<Character, String> {
        let character = self.characters.iter_mut().find(|c| c.id == id)
            .ok_or_else(|| format!("Character with id {} not found", id))?;

        if let Some(n) = name { character.name = n; }
        if let Some(d) = description { character.description = d; }
        if let Some(a) = aliases { character.aliases = a; }
        if let Some(t) = traits { character.traits = t; }
        if let Some(n) = notes { character.notes = n; }

        Ok(character.clone())
    }

    /// Remove a character by id
    pub fn remove_character(&mut self, id: usize) -> Result<Character, String> {
        let index = self.characters.iter().position(|c| c.id == id)
            .ok_or_else(|| format!("Character with id {} not found", id))?;

        // Also remove relationships involving this character
        self.relationships.retain(|r| r.character_a_id != id && r.character_b_id != id);

        Ok(self.characters.remove(index))
    }

    /// Get a character by id
    pub fn get_character(&self, id: usize) -> Option<&Character> {
        self.characters.iter().find(|c| c.id == id)
    }

    /// Find a character by name (exact match or alias)
    pub fn find_character_by_name(&self, name: &str) -> Option<&Character> {
        self.characters.iter().find(|c| {
            c.name == name || c.aliases.iter().any(|a| a == name)
        })
    }

    /// Add a new relationship
    pub fn add_relationship(&mut self, character_a_id: usize, character_b_id: usize, relationship_type: String, description: String) -> Result<Relationship, String> {
        // Verify both characters exist
        if self.get_character(character_a_id).is_none() {
            return Err(format!("Character with id {} not found", character_a_id));
        }
        if self.get_character(character_b_id).is_none() {
            return Err(format!("Character with id {} not found", character_b_id));
        }

        let relationship = Relationship {
            id: self.next_relationship_id,
            character_a_id,
            character_b_id,
            relationship_type,
            description,
        };
        self.next_relationship_id += 1;
        self.relationships.push(relationship.clone());
        Ok(relationship)
    }

    /// Update a relationship
    pub fn update_relationship(&mut self, id: usize, relationship_type: Option<String>, description: Option<String>) -> Result<Relationship, String> {
        let relationship = self.relationships.iter_mut().find(|r| r.id == id)
            .ok_or_else(|| format!("Relationship with id {} not found", id))?;

        if let Some(t) = relationship_type { relationship.relationship_type = t; }
        if let Some(d) = description { relationship.description = d; }

        Ok(relationship.clone())
    }

    /// Remove a relationship by id
    pub fn remove_relationship(&mut self, id: usize) -> Result<Relationship, String> {
        let index = self.relationships.iter().position(|r| r.id == id)
            .ok_or_else(|| format!("Relationship with id {} not found", id))?;
        Ok(self.relationships.remove(index))
    }

    /// Get all relationships for a specific character
    pub fn get_character_relationships(&self, character_id: usize) -> Vec<&Relationship> {
        self.relationships.iter()
            .filter(|r| r.character_a_id == character_id || r.character_b_id == character_id)
            .collect()
    }

    /// Set a key-value entry
    pub fn set_kv(&mut self, key: String, value: String, category: String, notes: String) {
        // Remove existing entry with same key
        self.key_value_store.retain(|e| e.key != key);

        self.key_value_store.push(KeyValueEntry {
            key,
            value,
            category,
            notes,
        });
    }

    /// Get a key-value entry
    pub fn get_kv(&self, key: &str) -> Option<&KeyValueEntry> {
        self.key_value_store.iter().find(|e| e.key == key)
    }

    /// Delete a key-value entry
    pub fn delete_kv(&mut self, key: &str) -> Option<KeyValueEntry> {
        let index = self.key_value_store.iter().position(|e| e.key == key);
        index.map(|i| self.key_value_store.remove(i))
    }

    /// List key-value entries by category
    pub fn list_kv_by_category(&self, category: &str) -> Vec<&KeyValueEntry> {
        self.key_value_store.iter()
            .filter(|e| e.category == category)
            .collect()
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_default() {
        let workspace = Workspace::default();
        assert!(workspace.characters.is_empty());
        assert!(workspace.relationships.is_empty());
        assert!(workspace.key_value_store.is_empty());
    }

    #[test]
    fn test_add_character() {
        let mut workspace = Workspace::new();
        let char = workspace.add_character("Alice".to_string(), "Main protagonist".to_string());
        assert_eq!(char.id, 0);
        assert_eq!(char.name, "Alice");
        assert_eq!(workspace.characters.len(), 1);
    }

    #[test]
    fn test_add_relationship() {
        let mut workspace = Workspace::new();
        workspace.add_character("Alice".to_string(), "desc".to_string());
        workspace.add_character("Bob".to_string(), "desc".to_string());

        let rel = workspace.add_relationship(0, 1, "friend".to_string(), "Best friends".to_string()).unwrap();
        assert_eq!(rel.id, 0);
        assert_eq!(workspace.relationships.len(), 1);
    }

    #[test]
    fn test_json_roundtrip() {
        let mut workspace = Workspace::new();
        workspace.add_character("Alice".to_string(), "desc".to_string());
        workspace.set_kv("theme".to_string(), "adventure".to_string(), "plot".to_string(), "".to_string());

        let json = workspace.to_json();
        let parsed = Workspace::from_json(&json).unwrap();
        assert_eq!(parsed.characters.len(), 1);
        assert_eq!(parsed.key_value_store.len(), 1);
    }
}