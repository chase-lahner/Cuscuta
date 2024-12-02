use std::vec::Vec;
use crate::cuscuta_resources::*;
use crate::ui::CarnageBar;
use bevy::prelude::*;

pub enum Room_Attributes{
    Room_Size,
    Inner_Walls,
    Enemy_Count,
    Enemy_Type,
    Item_Count,
}

impl Room_Attributes {
    pub fn get_preset_matrix() -> [Vec<Vec<f32>>; 5] {
        [
            vec![
                // Room_Size: Large, Medium, Small
                vec![0.2, 0.3, 0.5],
                vec![0.25, 0.5, 0.25],
                vec![0.4, 0.5, 0.1],
            ],
            vec![
                // Inner_Walls: Some, Little, None
                vec![0.0, 0.35, 0.65],
                vec![0.25, 0.5, 0.25],
                vec![0.10, 0.8, 0.10],
            ],
            vec![
                // Enemy_Count: Many, Some, Few
                vec![0.0, 0.35, 0.65],
                vec![0.25, 0.5, 0.25],
                vec![0.10, 0.8, 0.10],
            ],
            vec![
                // Enemy_Type: Stealth, Both, Carnage
                vec![0.0, 0.35, 0.65],
                vec![0.25, 0.5, 0.25],
                vec![0.10, 0.8, 0.10],
            ],
            vec![
                // Item_Count: Some, Little, None
                vec![0.0, 0.35, 0.65],
                vec![0.25, 0.5, 0.25],
                vec![0.10, 0.8, 0.10],
            ],
        ]
    }

    pub fn get_matrix_for_attribute(attribute: Room_Attributes) -> Vec<Vec<f32>> {
        let all_vectors = Room_Attributes::get_preset_matrix();
        match attribute {
            Room_Attributes::Room_Size => all_vectors[0].clone(),
            Room_Attributes::Inner_Walls => all_vectors[1].clone(),
            Room_Attributes::Enemy_Count => all_vectors[2].clone(),
            Room_Attributes::Enemy_Type => all_vectors[3].clone(),
            Room_Attributes::Item_Count => all_vectors[4].clone(),
        }
    }
}

#[derive(Resource)]
pub struct LastAttributeArray {
    pub attributes: [u8; 5], 
}

impl LastAttributeArray {
    // Constructor to initialize all values to 0 (default to "high")
    pub fn new() -> Self {
        Self { attributes: [1; 5] }
    }

    // Method to set a specific attribute
    pub fn set_attribute(&mut self, index: usize, value: u8) {
        if index < self.attributes.len() && value <= 2 {
            self.attributes[index] = value;
        } else {
            println!("Invalid index or value!");
        }
    }

    // Method to get the value of a specific attribute
    pub fn get_attribute(&self, index: usize) -> Option<u8> {
        self.attributes.get(index).copied()
    }
}

#[derive(Resource)]
pub struct NextAttributeArray {
    pub attributes: [u8; 5], 
}

impl NextAttributeArray {
    // Constructor to initialize all values to 0 (default to "high")
    pub fn new() -> Self {
        Self { attributes: [0; 5] }
    }

    // Method to set a specific attribute
    pub fn set_next_attribute(&mut self, index: usize, value: u8) {
        if index < self.attributes.len() && value <= 2 {
            self.attributes[index] = value;
        } else {
            println!("Invalid index or value!");
        }
    }

    // Method to get the value of a specific attribute
    pub fn get_attribute(&self, index: usize) -> Option<u8> {
        self.attributes.get(index).copied()
    }
}

//Function to skew our preset transition matrices towards carnage or stealth. 
pub fn Skew(input_matrix: Vec<Vec<f32>>, carnage_percent:f32) -> Vec<Vec<f32>> {
    //skew towards carnage rooms 
    let high_carnage_vec = vec![0.05,0.10,0.85];
    //skew towards stealth rooms 
    let low_carnage_vec = vec![0.85,0.10,0.05];

    //chooses which vec to skew or if to skew at all
    let skew_vec = if carnage_percent == 0.5 {
        return input_matrix; 
    } else if carnage_percent < 0.5 {
        low_carnage_vec
    } else {
        high_carnage_vec
    };

    //flag for which calculation to do
    let up_flag = if carnage_percent < 0.5 { 0 } else { 1 };

    //initialize output matrix to return later
    let mut skewed_matrix = vec![vec![0.; 3]; 3];

    if up_flag == 0{
        for (i, row) in input_matrix.iter().enumerate() {
            for (j, &value) in row.iter().enumerate() {
                skewed_matrix[i][j] = (1.-2.*(carnage_percent))*skew_vec[j]+(2.*carnage_percent)*value;
            }
        }
    }else{
        for (i, row) in input_matrix.iter().enumerate() {
            for (j, &value) in row.iter().enumerate() {
                skewed_matrix[i][j] = (1.-2.*(carnage_percent-0.5))*value+(2.*(carnage_percent-0.5))*skew_vec[j];
            }
        }
    }
    skewed_matrix
}









