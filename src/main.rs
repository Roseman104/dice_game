use rand::Rng;

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

// Player's turn function, returns points scored in this turn
fn player_turn(player_name: &str, own_score: u32, opponent_score: u32, winning_score: u32) -> u32 {
    let mut total_turn_score = 0;
    let mut remaining_dice = 6;

    println!("\n{}'s turn begins!", player_name);

    loop {
        // Roll dice
        let mut rolls = Vec::new();
        for _ in 0..remaining_dice {
            rolls.push(roll_dice());
        }

        println!("{} rolled: {:?}", player_name, rolls);

        // Count dice
        let mut counts = count_dice(&rolls);

        // Score dice and explain what was scored
        let (score, dice_used) = score_dice_verbose(&mut counts);

        if score == 0 {
            println!("💥 No scoring dice! {} loses all points for this turn.", player_name);
            return 0; // Bust, return zero points
        }

        println!("Points scored this roll: {}", score);
        println!("Dice used for scoring: {}", dice_used);

        // Update turn score
        total_turn_score += score;

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

        // Check if opponent is close to winning and player is far behind
        let opponent_close = opponent_score >= winning_score - 400;
        let player_far_behind = opponent_score >= own_score + 900;

        if opponent_close && player_far_behind {
            println!(
                "⚔️ {} is far behind and opponent is near winning — taking risks!",
                player_name
            );
            // In this mode, we ignore banking at 400 and always roll (unless bust)
        } else if remaining_dice == 6 {
            println!("🔥 {} has hot dice and chooses to roll again!", player_name);
            continue; // Roll again because of hot dice
        } else if total_turn_score >= 400 {
            println!("🏦 {} decides to bank the points: {}", player_name, total_turn_score);
            break;
        } else {
            println!("➕ {} decides to roll again!", player_name);
        }

        // In "desperate mode", we don't bank even if 400+ unless hot dice are used up
    }

    total_turn_score
}



fn main() {
    let mut player1_score = 0;
    let mut player2_score = 0;
    let winning_score = 4000;
    let mut turn = 1;

    println!("🎲 Welcome to the Dice Game! First to {} points wins! 🎲", winning_score);

    loop {
        println!("\n========== Turn {} ==========", turn);

        // Player 1's turn
        let p1_points = player_turn("Player 1", player1_score, player2_score, winning_score);
        player1_score += p1_points;
        println!("💰 Player 1 total score: {}", player1_score);

        // Check if Player 1 has won
        if player1_score >= winning_score {
            println!("\n🏆 Player 1 WINS with {} points! 🏆", player1_score);
            println!("\n\t Player 2 Final score: {} points", player2_score);
            break;
        }

        // Player 2's turn
        let p2_points = player_turn("Player 2", player2_score, player1_score, winning_score);
        player2_score += p2_points;
        println!("💰 Player 2 total score: {}", player2_score);

        // Check if Player 2 has won
        if player2_score >= winning_score {
            println!("\n🏆 Player 2 WINS with {} points! 🏆", player2_score);
            println!("\n\t Player 1 Final score: {} points", player1_score);
            break;
        }

        // Print both scores after each full round
        println!("Scores => Player 1: {} | Player 2: {}", player1_score, player2_score);

        turn += 1;
    }

    println!("\n🎉 Game Over! Thanks for playing! 🎉");
}
