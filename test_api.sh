#!/bin/bash

HOST="${1:-http://localhost:8080}"

echo "🧪 Test de l'API Lexique3"
echo "📡 Serveur : $HOST"
echo ""

GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

test_endpoint() {
    local name="$1"
    local url="$2"
    
    echo -n "Testing $name... "
    
    response=$(curl -s -w "\n%{http_code}" "$url")
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)
    
    if [ "$http_code" -eq 200 ]; then
        echo -e "${GREEN}✓${NC} (HTTP $http_code)"
        echo "$body" | head -c 200
    else
        echo -e "${RED}✗${NC} (HTTP $http_code)"
    fi
    echo ""
}

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1️⃣  HEALTH CHECK"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
test_endpoint "Health" "$HOST/health"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "2️⃣  STATISTIQUES"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
test_endpoint "Stats" "$HOST/stats"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "3️⃣  RECHERCHES"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
test_endpoint "Recherche 'chien'" "$HOST/search?ortho=chien"
test_endpoint "Recherche 'bonjour'" "$HOST/search?ortho=bonjour"
test_endpoint "Catégorie NOM" "$HOST/search?cgram=NOM&limit=3"

echo -e "${GREEN}✅ Tests terminés${NC}"