use rand::Rng;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug)]
struct Player {
    name: String,
    score: u32,
}



// Enum to classify roll type
#[derive(Debug)]
enum RollType {
    Interesting,
    Boring,
}

// Struct to track stats
#[derive(Default)]
struct RollStats {
    interesting: u32,
    boring: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct GameResult {
    game_id: Uuid,
    player1_score: u32,
    player2_score: u32,
    winner: String,
    interesting_rolls: u32,
    boring_rolls: u32,
    turns: Vec<TurnResult>, // Collect all turns
}


#[derive(Debug, Serialize, Deserialize)]
struct TurnResult {
    turn_id: Uuid,
    turn_number: u32,
    player: String,
    points_scored: u32,
    interesting: bool,
    rolls: Vec<Vec<u8>>, // Sequence of rolls
}


// Roll one die
fn roll_dice() -> u8 {
    let mut rng = rand::thread_rng();
    rng.gen_range(1..=6)
}

// Count occurrences of each dice face
fn count_dice(rolls: &Vec<u8>) -> [u8; 7] {
    let mut counts = [0; 7];
    for &roll in rolls {
        counts[roll as usize] += 1;
    }
    counts
}

fn score_dice_verbose(counts: &mut [u8; 7]) -> (u32, u8) {
    let mut score = 0;
    let mut dice_used = 0;

    println!("Scoring breakdown:");

    let mut pair_count = 0;
    let mut triplet_count = 0;

    // Check for special combinations first

    // Check for straight (1-6)
    if (1..=6).all(|num| counts[num] == 1) {
        println!("Straight (1-6) => 1500 points");
        score += 1500;
        dice_used += 6;
        // Reset counts since all dice are used
        for num in 1..=6 {
            counts[num] = 0;
        }
        return (score, dice_used);
    }

    // Check for three pairs
    for num in 1..=6 {
        if counts[num] == 2 {
            pair_count += 1;
        }
    }
    if pair_count == 3 {
        println!("Three pairs => 1500 points");
        score += 1500;
        dice_used += 6;
        for num in 1..=6 {
            counts[num] = 0;
        }
        return (score, dice_used);
    }

    // Check for two triplets
    for num in 1..=6 {
        if counts[num] == 3 {
            triplet_count += 1;
        }
    }
    if triplet_count == 2 {
        println!("Two triplets => 2500 points");
        score += 2500;
        dice_used += 6;
        for num in 1..=6 {
            counts[num] = 0;
        }
        return (score, dice_used);
    }

    // Check for six, five, four of a kind
    for num in 1..=6 {
        match counts[num] {
            6 => {
                println!("Six {}s => 3000 points", num);
                score += 3000;
                dice_used += 6;
                counts[num] = 0;
                return (score, dice_used);
            }
            5 => {
                println!("Five {}s => 2000 points", num);
                score += 2000;
                dice_used += 5;
                counts[num] = 0;
            }
            4 => {
                println!("Four {}s => 1000 points", num);
                score += 1000;
                dice_used += 4;
                counts[num] = 0;
            }
            _ => {}
        }
    }

    // Standard triples (if not already scored as part of special sets)
    for num in 1..=6 {
        if counts[num] >= 3 {
            if num == 1 {
                println!("Three 1s => 1000 points");
                score += 1000;
            } else {
                println!("Three {}s => {} points", num, num * 100);
                score += (num as u32) * 100;
            }
            counts[num] -= 3;
            dice_used += 3;
        }
    }

    // Score remaining 1's and 5's
    if counts[1] > 0 {
        println!("{} single 1(s) => {} points", counts[1], counts[1] * 100);
        score += (counts[1] as u32) * 100;
        dice_used += counts[1];
    }

    if counts[5] > 0 {
        println!("{} single 5(s) => {} points", counts[5], counts[5] * 50);
        score += (counts[5] as u32) * 50;
        dice_used += counts[5];
    }

    (score, dice_used)
}


// Calculate probability of scoring on next roll with remaining dice
fn probability_of_scoring(remaining_dice: u8) -> f64 {
    if remaining_dice == 0 {
        return 1.0; // If all dice scored, hot dice, 100% chance to roll again
    }
    let prob_no_score = (4.0_f64 / 6.0_f64).powi(remaining_dice as i32); // Explicit f64
    1.0 - prob_no_score
}

// Function to classify a dice roll
fn classify_roll(counts: &[u8; 7], remaining_dice: u8) -> RollType {
    // Check for "interesting" combinations
    let mut pair_count = 0;
    let mut triplet_count = 0;

    // Check for straight (1-6)
    if (1..=6).all(|num| counts[num] == 1) {
        return RollType::Interesting;
    }

    // Check for three pairs
    for num in 1..=6 {
        if counts[num] == 2 {
            pair_count += 1;
        }
    }
    if pair_count == 3 {
        return RollType::Interesting;
    }

    // Check for two triplets
    for num in 1..=6 {
        if counts[num] == 3 {
            triplet_count += 1;
        }
    }
    if triplet_count == 2 {
        return RollType::Interesting;
    }

    // Check for four, five, six of a kind
    for num in 1..=6 {
        if counts[num] >= 4 {
            return RollType::Interesting;
        }
    }

    // Check for regular triples (excluding single 1s and 5s)
    for num in 1..=6 {
        if counts[num] == 3 {
            return RollType::Interesting;
        }
    }

    // Check for "hot dice" (used all dice in scoring)
    if remaining_dice == 0 {
        return RollType::Interesting;
    }

    // Otherwise, only scoring 1's or 5's
    RollType::Boring
}

fn player_turn(
    game_id: Uuid,
    turn_number: u32,
    player_name: &str,
    own_score: u32,
    opponent_score: u32,
    winning_score: u32,
    stats: &mut RollStats,
) -> TurnResult {
    let mut total_turn_score = 0;
    let mut remaining_dice = 6;
    let mut rolls_history: Vec<Vec<u8>> = Vec::new(); // ✅ Explicit type for roll history
    let turn_id = Uuid::new_v4(); // Unique Turn ID

    println!("\n{}'s turn begins (Game ID: {}, Turn ID: {})", player_name, game_id, turn_id);

    loop {
        // Roll dice
        let mut rolls = Vec::new();
        for _ in 0..remaining_dice {
            rolls.push(roll_dice());
        }

        println!("{} rolled: {:?}", player_name, rolls);

        rolls_history.push(rolls.clone()); // ✅ Save the current roll into history

        // Count dice (original roll)
        let counts = count_dice(&rolls);

        // Now pass mutable copy for scoring
        let mut counts_for_scoring = counts;

        let (score, dice_used) = score_dice_verbose(&mut counts_for_scoring);

        // ✅ Classify roll BEFORE modifying counts for scoring
        let roll_type = classify_roll(&counts, dice_used); // ✅ Now passing 3 parameters as expected
        match roll_type {
            RollType::Interesting => {
                println!("✨ This was an **interesting** roll!");
                stats.interesting += 1;
            }
            RollType::Boring => {
                println!("😐 This was a **boring** roll.");
                stats.boring += 1;
            }
        }

        if score == 0 {
            println!("💥 No scoring dice! {} loses all points for this turn.", player_name);
            break; // Bust, end turn with zero points
        }

        println!("Points scored this roll: {}", score);
        println!("Dice used for scoring: {}", dice_used);

        // Update turn score
        total_turn_score += score;

        // Calculate points needed to win
        let points_needed_to_win = if own_score + total_turn_score >= winning_score {
            0
        } else {
            winning_score - (own_score + total_turn_score)
        };

        println!("🏁 {} needs {} more points to win.", player_name, points_needed_to_win);

        // Remaining dice calculation
        remaining_dice -= dice_used;
        if remaining_dice == 0 {
            println!("🔥 Hot dice! All dice scored, {} gets to roll all 6 dice again.", player_name);
            remaining_dice = 6;
        } else {
            println!("🎲 Dice left to roll: {}", remaining_dice);
        }

        // Calculate and display probability of scoring on next roll
        let prob_score = probability_of_scoring(remaining_dice);
        println!(
            "🎰 Probability of scoring on next roll with {} dice: {:.2}%",
            remaining_dice,
            prob_score * 100.0
        );

        // Check strategic conditions:
        let opponent_close = opponent_score >= winning_score - 400;
        let player_far_behind = opponent_score >= own_score + 900;
        let player_well_ahead = own_score >= opponent_score + 900;
        let player_safe_zone = own_score >= winning_score / 4;

        // ✅ If player is far behind and opponent is close to winning: take risks!
        if opponent_close && player_far_behind {
            println!(
                "⚔️ {} is far behind and opponent is near winning — taking risks!",
                player_name
            );
            // Risk mode: ignore banking unless hot dice
        }
        // ✅ If player is well ahead and in safe zone, play cautiously
        else if player_well_ahead && player_safe_zone {
            println!(
                "🛡️ {} is ahead by a comfortable margin and will play safely.",
                player_name
            );
            println!("🏦 {} decides to bank the points: {}", player_name, total_turn_score);
            break;
        }
        // ✅ If close to winning (e.g., need < 500 to win), play safe and bank what you get
        else if points_needed_to_win <= 500 {
            println!(
                "🏁 {} is close to winning and decides to bank the points to get closer to the goal.",
                player_name
            );
            println!("🏦 {} banks {} points.", player_name, total_turn_score);
            break
              ;
        }
        // ✅ Always reroll hot dice
        else if remaining_dice == 6 {
            println!("🔥 {} has hot dice and chooses to roll again!", player_name);
            continue; // Roll again because of hot dice
        }
        // ✅ Normal bank if 400+ points
        else if total_turn_score >= 400 {
            println!("🏦 {} decides to bank the points: {}", player_name, total_turn_score);
            break;
        }
        // ✅ Otherwise, continue rolling to build points
        else {
            println!("➕ {} decides to roll again!", player_name);
        }
    }

    TurnResult {
        turn_id,
        turn_number,
        player: player_name.to_string(),
        points_scored: total_turn_score,
        interesting: total_turn_score > 0, // or any condition you define
        rolls: rolls_history,
    }
}







fn main() {
    let mut player1 = Player { name: "Player 1".to_string(), score: 0 };
    let mut player2 = Player { name: "Player 2".to_string(), score: 0 };
    let winning_score = 4000;
    let mut turn = 1;
    let mut turns: Vec<TurnResult> = Vec::new(); // To collect all turns
    let mut turn_counter = 1;

    // Initialize stats tracker
    let mut stats = RollStats::default();

    let game_id = Uuid::new_v4();
    println!("🎮 Starting game with ID: {}", game_id);

    println!("🎲 Welcome to Farkle or 'the Dice Game'! First to {} points wins! 🎲", winning_score);

    loop {
        println!("\n========== Turn {} ==========", turn);

        // Player 1's turn
        let p1_turn = player_turn(
            game_id,
            turn_counter,
            &player1.name,
            player1.score,
            player2.score,
            winning_score,
            &mut stats,
        );
        player1.score += p1_turn.points_scored; // ✅ Access the points inside TurnResult
        turns.push(p1_turn);

        println!("💰 {} total score: {}", player1.name, player1.score);

        // Check if Player 1 has won
        if player1.score >= winning_score {
            println!("\n🏆 {} WINS with {} points! 🏆", player1.name, player1.score);
            println!("\n\t {} Final score: {} points", player2.name, player2.score);
            break;
        }

        // Player 2's turn
        let p2_turn = player_turn(
            game_id,
            turn_counter,
            &player2.name,
            player2.score,
            player1.score,
            winning_score,
            &mut stats,
        );
        player2.score += p2_turn.points_scored; // ✅ Access the points inside TurnResult
        turns.push(p2_turn);

        println!("💰 {} total score: {}", player2.name, player2.score);

        // Check if Player 2 has won
        if player2.score >= winning_score {
            println!("\n🏆 {} WINS with {} points! 🏆", player2.name, player2.score);
            println!("\n\t {} Final score: {} points", player1.name, player1.score);
            break;
        }

        // Print both scores after each full round
        println!("Scores => {}: {} | {}: {}", player1.name, player1.score, player2.name, player2.score);

        turn_counter += 1; // ✅ Increment turn counter only after both players' turns
        turn += 1;
    }

    // Show roll stats at the end
    println!("\n🎉 Game Over! Thanks for playing! 🎉");
    println!("\n📊 Roll Statistics:");
    println!("Interesting rolls: {}", stats.interesting);
    println!("Boring rolls: {}", stats.boring);

    println!("\n 🎮 Ending game: {}\n", game_id);

    // Optionally: Print all turns for review
    println!("\nTurn Results:");
    for t in turns {
        println!("{:?}", t);
    }
}

