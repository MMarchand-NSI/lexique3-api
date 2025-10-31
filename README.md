# API REST Lexique3

API REST haute performance en Rust pour interroger la base de données Lexique3.

## 🚀 Démarrage rapide

### Local
```bash
# Installer Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Lancer l'API
cargo run --release
```

### Test
```bash
# Health check
curl http://localhost:8080/health

# Stats
curl http://localhost:8080/stats | jq

# Recherche
curl "http://localhost:8080/search?ortho=chien" | jq
```

## 📡 Endpoints

### `GET /health`
Vérification de santé.

### `GET /stats`
Statistiques sur les données.

**Réponse:**
```json
{
  "total_entries": 142694,
  "unique_lemmes": 35000,
  "unique_phonemes": 38000
}
```

### `GET /search`
Recherche dans Lexique3.

**Paramètres:**
- `ortho` : recherche par orthographe
- `lemme` : recherche par lemme
- `phon` : recherche par phonologie
- `cgram` : catégorie grammaticale (NOM, VER, ADJ, etc.)
- `min_freq` : fréquence minimale
- `limit` : nombre max de résultats (défaut: 100, max: 1000)

**Exemples:**
```bash
curl "http://localhost:8080/search?ortho=chien"
curl "http://localhost:8080/search?cgram=NOM&min_freq=10&limit=50"
curl "http://localhost:8080/search?phon=ʃjɛ̃"
```

## 📥 Données Lexique3

Pour utiliser les vraies données:
```bash
wget http://www.lexique.org/databases/Lexique383/Lexique383.tsv
cargo run --release
```

## 🚀 Déploiement sur Fly.io
```bash
# Installer flyctl
curl -L https://fly.io/install.sh | sh

# Se connecter
flyctl auth login

# Déployer
flyctl launch
flyctl deploy

# Tester
curl https://lexique3-api.fly.dev/health
```

## 📊 Performances

- **Démarrage**: 1-3 sec
- **RAM**: 150-250 MB (Lexique3 complet)
- **Latence**: < 1 ms par requête

## 📝 Licence

Les données Lexique3 sont sous licence CC-BY-SA.

**Citation:**
> New, B., Pallier, C., Brysbaert, M., Ferrand, L. (2004) Lexique 2 : A New French Lexical Database. Behavior Research Methods, Instruments, & Computers, 36 (3), 516-524.