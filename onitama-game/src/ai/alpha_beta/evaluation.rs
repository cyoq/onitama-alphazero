use crate::{
    common::{count_bits, get_bit, get_msb},
    game::{
        move_result::MoveResult,
        player_color::PlayerColor,
        r#move::Move,
        state::{State, BLUE_TEMPLE, RED_TEMPLE},
    },
};

// PST is taken from: https://github.com/maxbennedich/onitama/blob/master/src/main/java/onitama/ai/evaluation/PieceSquareTables.java#L10
const PIECE_SQUARE_TABLE: [i32; 25] = [
    0, 4, 8, 4, 0, //
    4, 8, 12, 8, 4, //
    8, 12, 16, 12, 8, //
    4, 8, 12, 8, 4, //
    0, 4, 8, 4, 0,
];

#[derive(Debug)]
pub struct Evaluation;

impl Evaluation {
    pub fn evaluate(
        state: &State,
        player_color: PlayerColor,
        move_result: &Option<MoveResult>,
    ) -> i32 {
        let mut sign = 1;

        let enemy_color = player_color.enemy();
        if player_color == PlayerColor::Blue {
            sign = -1;
        }

        if let Some(move_result) = move_result {
            if *move_result == MoveResult::BlueWin || *move_result == MoveResult::RedWin {
                return -sign * 10000;
            }
        }

        let (enemy_temple, my_temple) = match player_color {
            PlayerColor::Red => (BLUE_TEMPLE, RED_TEMPLE),
            PlayerColor::Blue => (RED_TEMPLE, BLUE_TEMPLE),
        };

        let my_king = state.kings[player_color as usize];
        let enemy_king = state.kings[enemy_color as usize];
        let my_king_pos = get_msb(my_king);
        let enemy_king_pos = get_msb(enemy_king);

        let my_pawns = state.pawns[player_color as usize];
        let enemy_pawns = state.pawns[enemy_color as usize];

        // 10 and 10000 are just arbitrary numbers to signalize the significance of each piece
        let my_piece_score_sum = (count_bits(my_pawns) * 10 + count_bits(my_king) * 10000) as i32;
        let enemy_piece_score_sum =
            (count_bits(enemy_pawns) * 10 + count_bits(enemy_king) * 10000) as i32;

        // how far king is from the enemy temple
        let enemy_distance_to_temple = Self::distance(enemy_king_pos, my_temple as u32);
        let my_distance_to_temple = Self::distance(my_king_pos, enemy_temple as u32);

        let mut me_close_to_enemy_king = 0;
        let mut enemies_close_to_my_king = 0;

        let mut my_piece_square = 0;
        let mut enemy_piece_square = 0;

        for n in 0..25 {
            let my_pawn = get_bit(my_pawns, n);
            let enemy_pawn = get_bit(enemy_pawns, n);

            if my_pawn == 1 {
                me_close_to_enemy_king += Self::distance(my_pawn, enemy_king_pos);
                my_piece_square += PIECE_SQUARE_TABLE[n];
            } else if enemy_pawn == 1 {
                enemies_close_to_my_king += Self::distance(enemy_pawn, my_king_pos);
                enemy_piece_square += PIECE_SQUARE_TABLE[n];
            }
        }

        sign * ((my_piece_score_sum - enemy_piece_score_sum)
            + (my_distance_to_temple - enemy_distance_to_temple)
            + (me_close_to_enemy_king - enemies_close_to_my_king)
            + (my_piece_square - enemy_piece_square))
    }

    fn distance(from: u32, to: u32) -> i32 {
        let (from_x, from_y) = Move::convert_to_2d(from);
        let (to_x, to_y) = Move::convert_to_2d(to);
        Self::euclidean_distance(from_x, from_y, to_x, to_y)
    }

    #[allow(dead_code)]
    #[inline]
    fn manhattan_distance(from_x: u32, from_y: u32, to_x: u32, to_y: u32) -> i32 {
        (to_x as i32 - from_x as i32).abs() + (to_y as i32 - from_y as i32).abs()
    }

    #[allow(dead_code)]
    #[inline]
    fn euclidean_distance(from_x: u32, from_y: u32, to_x: u32, to_y: u32) -> i32 {
        ((to_x as f32 - from_x as f32).powf(2.) + (to_y as f32 - from_y as f32).powf(2.)).sqrt()
            as i32
    }
}
