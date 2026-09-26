#!/usr/bin/env bash
set -e

CERT_NAME="${1:-Corvo Development}"
P12_PASS="${2:-corvo-developer}"

echo "=================================================="
echo "🔐 macOS 本地通用持久化代码签名证书配置"
echo "=================================================="

# 1. 检查是否已有同名代码签名证书
if security find-identity -v -p codesigning | grep -q "\"$CERT_NAME\""; then
    echo "✅ 已检测到有效的通用签名证书: \"$CERT_NAME\""
    echo "打包脚本将会自动使用该证书，重编后权限永不失效！"
    exit 0
fi

echo "正在为当前开发者生成通用代码签名证书: \"$CERT_NAME\"..."

TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

CNF_FILE="$TEMP_DIR/cert.cnf"
KEY_FILE="$TEMP_DIR/key.pem"
CERT_FILE="$TEMP_DIR/cert.pem"
P12_FILE="$TEMP_DIR/cert.p12"

# 2. 编写 OpenSSL 配置（指定 Code Signing 扩展）
cat > "$CNF_FILE" <<EOF
[req]
distinguished_name = req_distinguished_name
prompt = no
[req_distinguished_name]
CN = $CERT_NAME
[v3_ca]
keyUsage = critical, digitalSignature
extendedKeyUsage = critical, codeSigning
basicConstraints = critical, CA:true
EOF

# 3. 生成自签名根证书（有效期 10 年，3650 天）
openssl req -new -x509 -newkey rsa:2048 -nodes \
    -keyout "$KEY_FILE" -out "$CERT_FILE" -days 3650 \
    -config "$CNF_FILE" -extensions v3_ca >/dev/null 2>&1

# 4. 导出为兼容 macOS Keychain 的 PKCS#12 格式
openssl pkcs12 -export -inkey "$KEY_FILE" -in "$CERT_FILE" \
    -out "$P12_FILE" -passout "pass:$P12_PASS" -legacy >/dev/null 2>&1

# 5. 导入证书与私钥至用户钥匙串
KEYCHAIN="$HOME/Library/Keychains/login.keychain-db"
if [ ! -f "$KEYCHAIN" ]; then
    KEYCHAIN="$HOME/Library/Keychains/login.keychain"
fi

echo "📦 正在导入证书至钥匙串: $KEYCHAIN ..."
security import "$P12_FILE" -k "$KEYCHAIN" -P "$P12_PASS" -T /usr/bin/codesign >/dev/null 2>&1

# 6. 将证书添加至代码签名信任列表（仅限当前用户域，无需修改系统核心）
echo "🛡️  正在配置信任策略 (针对代码签名)..."
security add-trusted-cert -r trustRoot -p codeSign -k "$KEYCHAIN" "$CERT_FILE" >/dev/null 2>&1 || true

echo "=================================================="
echo "🎉 通用证书配置成功！"
security find-identity -p codesigning "$KEYCHAIN" | grep "$CERT_NAME" || true
echo "=================================================="
echo "👉 后续编译任何项目均可直接复用该证书签名。"
echo "重新编译后，macOS 辅助功能等系统权限将永久保持有效，不再失效！"
