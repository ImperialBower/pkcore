use anyhow::Result;
use log::info;
use pkcore::analysis::gto::combos::Combos;
use pkcore::analysis::gto::vs::Versus;
use pkcore::analysis::store::db::hup::HUPResult;
use pkcore::arrays::two::Two;
use pkcore::play::board::Board;
use pkcore::GTO;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::str::FromStr;
use warp::Filter;

#[derive(Debug, Deserialize)]
struct GTORequest {
    player: String,
    villain: String,
    board: Option<String>,
    nuts: Option<bool>,
}

#[derive(Debug, Serialize)]
struct GTOResponse {
    player: String,
    villain: String,
    board: Option<String>,
    combo_pairs: String,
    villain_combo_pairs: String,
    results: GTOResults,
    flop_results: Option<String>,
    turn_results: Option<String>,
    elapsed_ms: u128,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct GTOResults {
    win_lose_draw: String,
    hup_results: Vec<String>,
}

async fn calculate_gto(request: GTORequest) -> Result<impl warp::Reply, Infallible> {
    let start_time = std::time::Instant::now();
    
    info!("Processing GTO request: {:?}", request);
    
    let response = match process_gto_calculation(request).await {
        Ok(mut resp) => {
            resp.elapsed_ms = start_time.elapsed().as_millis();
            resp
        }
        Err(e) => GTOResponse {
            player: "".to_string(),
            villain: "".to_string(),
            board: None,
            combo_pairs: "".to_string(),
            villain_combo_pairs: "".to_string(),
            results: GTOResults {
                win_lose_draw: "".to_string(),
                hup_results: vec![],
            },
            flop_results: None,
            turn_results: None,
            elapsed_ms: start_time.elapsed().as_millis(),
            error: Some(format!("Error: {}", e)),
        }
    };
    
    Ok(warp::reply::json(&response))
}

async fn process_gto_calculation(request: GTORequest) -> Result<GTOResponse> {
    // Parse player hand
    let player_hand = Two::from_str(&request.player)
        .map_err(|e| anyhow::anyhow!("Invalid player hand '{}': {}", request.player, e))?;
    
    // Parse villain range
    let villain_range = Combos::from_str(&request.villain)
        .map_err(|e| anyhow::anyhow!("Invalid villain range '{}': {}", request.villain, e))?;
    
    // Create solver
    let solver = if let Some(board_str) = &request.board {
        let board = Board::from_str(board_str)
            .map_err(|e| anyhow::anyhow!("Invalid board '{}': {}", board_str, e))?;
        Versus::new_with_board(player_hand, villain_range, board)
    } else {
        Versus::new(player_hand, villain_range)
    };
    
    // Get combo pairs
    let combo_pairs = solver.combo_pairs().to_string();
    let villain_combo_pairs = solver.villain.combo_pairs().to_string();
    
    // Connect to database (try to open, create empty response if fails)
    let conn_result = Connection::open("generated/hups.db");
    let (hup_results, win_lose_draw) = if let Ok(conn) = conn_result {
        let hups = solver.hups_at_deal(&conn);
        let hup_strings: Vec<String> = hups.values().map(|h| h.to_string()).collect();
        let combined_odds = Versus::combined_odds_at_deal(&hups.values().collect::<Vec<&HUPResult>>());
        (hup_strings, combined_odds.to_string())
    } else {
        (vec!["Database not available".to_string()], "Database not available".to_string())
    };
    
    // Calculate flop and turn results if board is present
    let (flop_results, turn_results) = if solver.has_board() {
        let flop_odds = solver.combined_odds_at_flop();
        let turn_odds = solver.combined_odds_at_turn();
        (Some(format!("FLOP: {}", flop_odds)), Some(format!("TURN: {}", turn_odds)))
    } else {
        (None, None)
    };
    
    Ok(GTOResponse {
        player: request.player,
        villain: request.villain,
        board: request.board,
        combo_pairs,
        villain_combo_pairs,
        results: GTOResults {
            win_lose_draw,
            hup_results,
        },
        flop_results,
        turn_results,
        elapsed_ms: 0, // Will be set by caller
        error: None,
    })
}

async fn serve_index() -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::html(INDEX_HTML))
}

const INDEX_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Poker GTO Calculator</title>
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background-color: #f5f5f5;
        }
        .container {
            background: white;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        h1 {
            color: #2c3e50;
            text-align: center;
            margin-bottom: 30px;
        }
        .form-group {
            margin-bottom: 20px;
        }
        label {
            display: block;
            margin-bottom: 5px;
            font-weight: bold;
            color: #34495e;
        }
        input[type="text"] {
            width: 100%;
            padding: 12px;
            border: 2px solid #ddd;
            border-radius: 5px;
            font-size: 16px;
            transition: border-color 0.3s;
        }
        input[type="text"]:focus {
            outline: none;
            border-color: #3498db;
        }
        .checkbox-group {
            display: flex;
            align-items: center;
        }
        input[type="checkbox"] {
            margin-right: 10px;
            transform: scale(1.2);
        }
        button {
            background-color: #3498db;
            color: white;
            padding: 12px 30px;
            border: none;
            border-radius: 5px;
            font-size: 16px;
            cursor: pointer;
            transition: background-color 0.3s;
        }
        button:hover {
            background-color: #2980b9;
        }
        button:disabled {
            background-color: #bdc3c7;
            cursor: not-allowed;
        }
        .results {
            margin-top: 30px;
            padding: 20px;
            background-color: #ecf0f1;
            border-radius: 5px;
            display: none;
        }
        .results.show {
            display: block;
        }
        .result-section {
            margin-bottom: 20px;
        }
        .result-section h3 {
            color: #2c3e50;
            margin-bottom: 10px;
        }
        .result-content {
            background: white;
            padding: 15px;
            border-radius: 5px;
            font-family: 'Courier New', monospace;
            white-space: pre-wrap;
            border-left: 4px solid #3498db;
        }
        .error {
            background-color: #e74c3c;
            color: white;
            padding: 15px;
            border-radius: 5px;
            margin-top: 20px;
        }
        .loading {
            text-align: center;
            color: #7f8c8d;
            margin-top: 20px;
        }
        .examples {
            background-color: #f8f9fa;
            padding: 15px;
            border-radius: 5px;
            margin-bottom: 20px;
            border-left: 4px solid #17a2b8;
        }
        .examples h4 {
            margin-top: 0;
            color: #495057;
        }
        .examples code {
            background-color: #e9ecef;
            padding: 2px 6px;
            border-radius: 3px;
            font-family: 'Courier New', monospace;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🃏 Poker GTO Calculator</h1>
        
        <div class="examples">
            <h4>Examples:</h4>
            <p><strong>Player:</strong> <code>K♠ K♥</code> or <code>KsKh</code></p>
            <p><strong>Villain:</strong> <code>66+,AJs+,KQs,AJo+,KQo</code></p>
            <p><strong>Board:</strong> <code>Kc 7h 2d</code> (optional)</p>
        </div>
        
        <form id="gtoForm">
            <div class="form-group">
                <label for="player">Player Hand:</label>
                <input type="text" id="player" name="player" placeholder="e.g., K♠ K♥" required>
            </div>
            
            <div class="form-group">
                <label for="villain">Villain Range:</label>
                <input type="text" id="villain" name="villain" placeholder="e.g., 66+,AJs+,KQs,AJo+,KQo" required>
            </div>
            
            <div class="form-group">
                <label for="board">Board (optional):</label>
                <input type="text" id="board" name="board" placeholder="e.g., Kc 7h 2d">
            </div>
            
            <div class="form-group">
                <div class="checkbox-group">
                    <input type="checkbox" id="nuts" name="nuts">
                    <label for="nuts">Calculate nuts</label>
                </div>
            </div>
            
            <button type="submit" id="calculateBtn">Calculate GTO</button>
        </form>
        
        <div id="loading" class="loading" style="display: none;">
            Calculating... This may take a moment.
        </div>
        
        <div id="error" class="error" style="display: none;"></div>
        
        <div id="results" class="results">
            <div class="result-section">
                <h3>Calculation Time</h3>
                <div id="elapsed" class="result-content"></div>
            </div>
            
            <div class="result-section">
                <h3>Player Combo Pairs</h3>
                <div id="playerCombos" class="result-content"></div>
            </div>
            
            <div class="result-section">
                <h3>Villain Combo Pairs</h3>
                <div id="villainCombos" class="result-content"></div>
            </div>
            
            <div class="result-section">
                <h3>Win/Lose/Draw Results</h3>
                <div id="winLoseDraw" class="result-content"></div>
            </div>
            
            <div class="result-section">
                <h3>HUP Results</h3>
                <div id="hupResults" class="result-content"></div>
            </div>
            
            <div id="flopSection" class="result-section" style="display: none;">
                <h3>Flop Results</h3>
                <div id="flopResults" class="result-content"></div>
            </div>
            
            <div id="turnSection" class="result-section" style="display: none;">
                <h3>Turn Results</h3>
                <div id="turnResults" class="result-content"></div>
            </div>
        </div>
    </div>

    <script>
        document.getElementById('gtoForm').addEventListener('submit', async function(e) {
            e.preventDefault();
            
            const calculateBtn = document.getElementById('calculateBtn');
            const loading = document.getElementById('loading');
            const results = document.getElementById('results');
            const error = document.getElementById('error');
            
            // Reset UI
            calculateBtn.disabled = true;
            loading.style.display = 'block';
            results.classList.remove('show');
            error.style.display = 'none';
            
            // Collect form data
            const formData = new FormData(e.target);
            const requestData = {
                player: formData.get('player'),
                villain: formData.get('villain'),
                board: formData.get('board') || null,
                nuts: formData.get('nuts') === 'on'
            };
            
            try {
                const response = await fetch('/api/gto', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify(requestData)
                });
                
                const data = await response.json();
                
                if (data.error) {
                    throw new Error(data.error);
                }
                
                // Display results
                document.getElementById('elapsed').textContent = `${data.elapsed_ms}ms`;
                document.getElementById('playerCombos').textContent = data.combo_pairs;
                document.getElementById('villainCombos').textContent = data.villain_combo_pairs;
                document.getElementById('winLoseDraw').textContent = data.results.win_lose_draw;
                document.getElementById('hupResults').textContent = data.results.hup_results.join('\n');
                
                // Show flop/turn results if available
                if (data.flop_results) {
                    document.getElementById('flopResults').textContent = data.flop_results;
                    document.getElementById('flopSection').style.display = 'block';
                } else {
                    document.getElementById('flopSection').style.display = 'none';
                }
                
                if (data.turn_results) {
                    document.getElementById('turnResults').textContent = data.turn_results;
                    document.getElementById('turnSection').style.display = 'block';
                } else {
                    document.getElementById('turnSection').style.display = 'none';
                }
                
                results.classList.add('show');
                
            } catch (err) {
                error.textContent = err.message;
                error.style.display = 'block';
            } finally {
                calculateBtn.disabled = false;
                loading.style.display = 'none';
            }
        });
    </script>
</body>
</html>
"#;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    info!("Starting Poker GTO Web Server...");
    
    // GET / -> index page
    let index = warp::path::end()
        .and(warp::get())
        .and_then(serve_index);
    
    // POST /api/gto -> GTO calculation
    let gto_api = warp::path!("api" / "gto")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(calculate_gto);
    
    // Combine routes
    let routes = index
        .or(gto_api)
        .with(warp::cors().allow_any_origin())
        .with(warp::log("web_gto"));
    
    // Start server
    info!("Server starting on http://0.0.0.0:3030");
    warp::serve(routes)
        .run(([0, 0, 0, 0], 3030))
        .await;
}