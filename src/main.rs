use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tower_http::cors::CorsLayer;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LexiqueEntry {
    ortho: String,
    phon: String,
    lemme: String,
    cgram: String,
    genre: String,
    nombre: String,
    freqlemfilms2: f32,
    freqlemlivres: f32,
    nbr_syll: u8,
}

#[derive(Clone)]
struct AppState {
    entries: Vec<LexiqueEntry>,
    ortho_index: HashMap<String, Vec<usize>>,
    lemme_index: HashMap<String, Vec<usize>>,
    phon_index: HashMap<String, Vec<usize>>,
}

impl AppState {
    fn new(entries: Vec<LexiqueEntry>) -> Self {
        let mut ortho_index = HashMap::new();
        let mut lemme_index = HashMap::new();
        let mut phon_index = HashMap::new();

        for (idx, entry) in entries.iter().enumerate() {
            ortho_index
                .entry(entry.ortho.to_lowercase())
                .or_insert_with(Vec::new)
                .push(idx);
            
            lemme_index
                .entry(entry.lemme.to_lowercase())
                .or_insert_with(Vec::new)
                .push(idx);
            
            phon_index
                .entry(entry.phon.clone())
                .or_insert_with(Vec::new)
                .push(idx);
        }

        Self {
            entries,
            ortho_index,
            lemme_index,
            phon_index,
        }
    }
}

#[derive(Debug, Deserialize)]
struct SearchParams {
    ortho: Option<String>,
    lemme: Option<String>,
    phon: Option<String>,
    cgram: Option<String>,
    min_freq: Option<f32>,
    limit: Option<usize>,
}

#[derive(Serialize)]
struct SearchResponse {
    count: usize,
    results: Vec<LexiqueEntry>,
}

#[derive(Serialize)]
struct StatsResponse {
    total_entries: usize,
    unique_lemmes: usize,
    unique_phonemes: usize,
}

async fn health_check() -> &'static str {
    "OK"
}

async fn stats(State(state): State<Arc<AppState>>) -> Json<StatsResponse> {
    Json(StatsResponse {
        total_entries: state.entries.len(),
        unique_lemmes: state.lemme_index.len(),
        unique_phonemes: state.phon_index.len(),
    })
}

async fn search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResponse>, StatusCode> {
    let limit = params.limit.unwrap_or(100).min(1000);
    
    let mut indices: Option<Vec<usize>> = None;
    
    if let Some(ortho) = &params.ortho {
        let ortho_lower = ortho.to_lowercase();
        if let Some(idx_list) = state.ortho_index.get(&ortho_lower) {
            indices = Some(idx_list.clone());
        } else {
            return Ok(Json(SearchResponse {
                count: 0,
                results: vec![],
            }));
        }
    }
    
    if let Some(lemme) = &params.lemme {
        let lemme_lower = lemme.to_lowercase();
        if let Some(idx_list) = state.lemme_index.get(&lemme_lower) {
            indices = match indices {
                Some(existing) => {
                    let intersection: Vec<usize> = existing
                        .into_iter()
                        .filter(|i| idx_list.contains(i))
                        .collect();
                    Some(intersection)
                }
                None => Some(idx_list.clone()),
            };
        } else {
            return Ok(Json(SearchResponse {
                count: 0,
                results: vec![],
            }));
        }
    }
    
    if let Some(phon) = &params.phon {
        if let Some(idx_list) = state.phon_index.get(phon) {
            indices = match indices {
                Some(existing) => {
                    let intersection: Vec<usize> = existing
                        .into_iter()
                        .filter(|i| idx_list.contains(i))
                        .collect();
                    Some(intersection)
                }
                None => Some(idx_list.clone()),
            };
        } else {
            return Ok(Json(SearchResponse {
                count: 0,
                results: vec![],
            }));
        }
    }
    
    let indices = indices.unwrap_or_else(|| (0..state.entries.len()).collect());
    
    let results: Vec<LexiqueEntry> = indices
        .into_iter()
        .filter_map(|i| {
            let entry = &state.entries[i];
            
            if let Some(cgram) = &params.cgram {
                if !entry.cgram.eq_ignore_ascii_case(cgram) {
                    return None;
                }
            }
            
            if let Some(min_freq) = params.min_freq {
                if entry.freqlemfilms2 < min_freq && entry.freqlemlivres < min_freq {
                    return None;
                }
            }
            
            Some(entry.clone())
        })
        .take(limit)
        .collect();
    
    let count = results.len();
    
    Ok(Json(SearchResponse { count, results }))
}

fn load_lexique_data() -> Result<Vec<LexiqueEntry>, Box<dyn std::error::Error>> {
    info!("Chargement des données Lexique3...");
    
    let lexique_path = "Lexique383.tsv";
    
    if std::path::Path::new(lexique_path).exists() {
        info!("Chargement depuis {}", lexique_path);
        
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .from_path(lexique_path)?;
        
        let mut entries = Vec::new();
        
        for (i, result) in rdr.records().enumerate() {
            let record = result?;
            
            if record.len() < 26 {
                continue;
            }
            
            let ortho = record.get(0).unwrap_or("").to_string();
            let phon = record.get(1).unwrap_or("").to_string();
            let lemme = record.get(2).unwrap_or("").to_string();
            let cgram = record.get(3).unwrap_or("").to_string();
            let genre = record.get(4).unwrap_or("").to_string();
            let nombre = record.get(5).unwrap_or("").to_string();
            
            let freqlemfilms2 = record.get(6)
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(0.0);
            
            let freqlemlivres = record.get(7)
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(0.0);
            
            let nbr_syll = record.get(24)
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(0);
            
            entries.push(LexiqueEntry {
                ortho,
                phon,
                lemme,
                cgram,
                genre,
                nombre,
                freqlemfilms2,
                freqlemlivres,
                nbr_syll,
            });
            
            if (i + 1) % 10000 == 0 {
                info!("Chargé {} entrées...", i + 1);
            }
        }
        
        info!("✓ Chargé {} entrées depuis Lexique3", entries.len());
        Ok(entries)
    } else {
        info!("⚠ Fichier {} non trouvé, utilisation de données de démo", lexique_path);
        info!("  Téléchargez depuis: http://www.lexique.org/databases/Lexique383/Lexique383.tsv");
        
        let entries = vec![
            LexiqueEntry {
                ortho: "chien".to_string(),
                phon: "ʃjɛ̃".to_string(),
                lemme: "chien".to_string(),
                cgram: "NOM".to_string(),
                genre: "m".to_string(),
                nombre: "s".to_string(),
                freqlemfilms2: 12.5,
                freqlemlivres: 15.3,
                nbr_syll: 1,
            },
            LexiqueEntry {
                ortho: "chat".to_string(),
                phon: "ʃa".to_string(),
                lemme: "chat".to_string(),
                cgram: "NOM".to_string(),
                genre: "m".to_string(),
                nombre: "s".to_string(),
                freqlemfilms2: 8.2,
                freqlemlivres: 11.7,
                nbr_syll: 1,
            },
            LexiqueEntry {
                ortho: "bonjour".to_string(),
                phon: "bɔ̃ʒuʁ".to_string(),
                lemme: "bonjour".to_string(),
                cgram: "NOM".to_string(),
                genre: "m".to_string(),
                nombre: "s".to_string(),
                freqlemfilms2: 45.3,
                freqlemlivres: 32.1,
                nbr_syll: 2,
            },
        ];
        
        info!("✓ Chargé {} entrées de démo", entries.len());
        Ok(entries)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();
    
    info!("Démarrage de l'API Lexique3");
    
    let entries = load_lexique_data()?;
    let state = Arc::new(AppState::new(entries));
    
    let app = Router::new()
        .route("/", get(|| async { "Lexique3 API - utilisez /search ou /stats" }))
        .route("/health", get(health_check))
        .route("/stats", get(stats))
        .route("/search", get(search))
        .layer(CorsLayer::permissive())
        .with_state(state);
    
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()?;
    
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    
    info!("Serveur en écoute sur {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}