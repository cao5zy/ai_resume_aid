#!/usr/bin/env bash
set -euo pipefail

# Smoke-test the AI Resume Aid backend API endpoints.
#
# Usage:
#   bash scripts/api-smoke.sh [base_url]
#
# Default base_url: http://localhost:3001/api
# (base_url should include the API prefix, e.g. http://host:port/api or http://host:port/v1)
#
# Requires: curl, jq
#   brew install jq        # macOS
#   sudo apt install jq    # Ubuntu/Debian

BASE_URL="${1:-http://localhost:3001/api}"
PASS=0
FAIL=0

check_json() {
  local label="$1"
  local response="$2"
  local expected_success="$3"

  if echo "$response" | jq -e ".success == $expected_success" >/dev/null 2>&1; then
    echo "  ✓ $label"
    PASS=$((PASS + 1))
  else
    echo "  ✗ $label"
    echo "    Response: $(echo "$response" | jq -c . 2>/dev/null || echo "$response")"
    FAIL=$((FAIL + 1))
  fi
}

echo "=== AI Resume Aid Smoke Test ==="
echo "Base URL: $BASE_URL"
echo ""

# ── 1. GET /config ────────────────────────────────────────────
echo "1. GET /config"
RESP=$(curl -s "$BASE_URL/config" 2>&1 || true)
check_json "config returns {success, data.upload_max_size_mb}" "$RESP" true

# ── 2. POST /optimize (JSON mode) ─────────────────────────────
echo "2. POST /optimize (JSON)"
RESP=$(curl -s -X POST "$BASE_URL/optimize" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "曾在工厂做质检工作3年，熟悉质量管控流程",
    "group": "disabled"
  }' 2>&1 || true)
check_json "optimize JSON returns success" "$RESP" true

# Check that optimized_text is present
if echo "$RESP" | jq -e '.data.optimized_text | length > 0' >/dev/null 2>&1; then
  echo "    ✓ optimized_text is non-empty"
  PASS=$((PASS + 1))
else
  echo "    ✗ optimized_text is empty or missing"
  FAIL=$((FAIL + 1))
fi

# ── 3. POST /search-jobs ──────────────────────────────────────
echo "3. POST /search-jobs"
RESP=$(curl -s -X POST "$BASE_URL/search-jobs" \
  -H "Content-Type: application/json" \
  -d '{
    "group": "disabled"
  }' 2>&1 || true)
check_json "search-jobs returns success" "$RESP" true

# ── 4. POST /export-pdf ──────────────────────────────────────
echo "4. POST /export-pdf"
RESP=$(curl -s -X POST "$BASE_URL/export-pdf" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "## 个人信息\n张三 | 5年质检经验\n\n## 工作经历\n- 负责生产线质量抽检\n- 使用质量管理工具分析缺陷率",
    "title": "测试简历"
  }' --output /tmp/api-smoke-test.pdf -w "%{http_code}" 2>&1 || true)

if [ "$RESP" = "200" ]; then
  SIZE=$(wc -c < /tmp/api-smoke-test.pdf 2>/dev/null || echo 0)
  if [ "$SIZE" -gt 10000 ]; then
    echo "  ✓ export-pdf returns 200 + PDF ($SIZE bytes)"
    PASS=$((PASS + 1))
  else
    echo "  ✗ export-pdf PDF too small ($SIZE bytes)"
    FAIL=$((FAIL + 1))
  fi
  rm -f /tmp/api-smoke-test.pdf
else
  echo "  ✗ export-pdf returned HTTP $RESP"
  FAIL=$((FAIL + 1))
fi

# ── 5. POST /export-pdf with CJK + bullet chars ──────────────
echo "5. POST /export-pdf (CJK + bullet)"
RESP=$(curl -s -X POST "$BASE_URL/export-pdf" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "### 工作经历\n#### 某某公司\n- 负责质量管控\n- 主导流程优化",
    "title": "CJK测试"
  }' --output /tmp/api-smoke-cjk-test.pdf -w "%{http_code}" 2>&1 || true)

if [ "$RESP" = "200" ]; then
  SIZE=$(wc -c < /tmp/api-smoke-cjk-test.pdf 2>/dev/null || echo 0)
  if [ "$SIZE" -gt 10000 ]; then
    echo "  ✓ CJK PDF returned 200 ($SIZE bytes)"
    PASS=$((PASS + 1))
  else
    echo "  ✗ CJK PDF too small ($SIZE bytes)"
    FAIL=$((FAIL + 1))
  fi
  rm -f /tmp/api-smoke-cjk-test.pdf
else
  echo "  ✗ CJK PDF returned HTTP $RESP"
  FAIL=$((FAIL + 1))
fi

# ── Summary ──────────────────────────────────────────────────────
echo ""
echo "=== Results: $PASS passed, $FAIL failed ==="
if [ "$FAIL" -gt 0 ]; then
  exit 1
fi
