use std::vec::Vec;
use crate::cuscuta_resources::*;
use crate::ui:CarnageBar;

pub enum Room_Attributes{
    Room_Size,
    Inner_Walls,
    Enemy_Count,
    Enemy_Type,
    Item_Count,
}

impl Room_Attributes {
    fn get_preset_vector(&self) -> Vec<Vec<f32>> {
        match self {
            Room_Attributes::Room_Size => vec![
                //Large, Medium, Small
                vec![0.2, 0.3, 0.5],
                vec![0.25, 0.5, 0.25],
                vec![0.4, 0.5, 0.1],
            ],
            Room_Attributes::Inner_Walls => vec![
                //Some, Little, None
                vec![0.0, 0.35, 0.65],
                vec![0.25, 0.5, 0.25],
                vec![0.10, 0.8, 0.10],
            ],
            Room_Attributes::Enemy_Count => vec![
                //Many, Some, Few
                vec![0.0, 0.35, 0.65],
                vec![0.25, 0.5, 0.25],
                vec![0.10, 0.8, 0.10],
            ],
            Room_Attributes::Enemy_Type => vec![
                //Stealth, Both, Carnage
                vec![0.0, 0.35, 0.65],
                vec![0.25, 0.5, 0.25],
                vec![0.10, 0.8, 0.10],
            ],
            Room_Attributes::Item_Count => vec![
                //Some, Little, None
                vec![0.0, 0.35, 0.65],
                vec![0.25, 0.5, 0.25],
                vec![0.10, 0.8, 0.10],
            ],
        }
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
                skewed_matrix[i][j] = (1-2*(carnage_percent))*skew_vec[j]+(2*carnage_percent)*value;
            }
        }
    }else{
        for (i, row) in input_matrix.iter().enumerate() {
            for (j, &value) in row.iter().enumerate() {
                skewed_matrix[i][j] = (1-2*(carnage_percent-0.5))*value+(2*(carnage_percent-0.5))*skew_vec[j];
            }
        }
    }
    skewed_matrix
}








