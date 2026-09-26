#!/usr/bin/env bash
set -e

CERT_NAME="${1:-Corvo Development}"
OUTPUT_FILE="${2:-Corvo_Development.p12}"
PASSWORD="${3:-corvo-developer}"

echo "=================================================="
echo "📤 导出通用代码签名证书 (.p12) 用于 CI/CD 打包机器"
echo "=================================================="

KEYCHAIN="$HOME/Library/Keychains/login.keychain-db"
if [ ! -f "$KEYCHAIN" ]; then
    KEYCHAIN="$HOME/Library/Keychains/login.keychain"
fi

if ! security find-identity -p codesigning "$KEYCHAIN" | grep -q "\"$CERT_NAME\""; then
    echo "❌ 错误: 未在钥匙串中找到名为 \"$CERT_NAME\" 的证书。"
    echo "请先运行 ./scripts/setup_certificate.sh 生成本地证书。"
    exit 1
fi

echo "正在从钥匙串导出 \"$CERT_NAME\" 到: $OUTPUT_FILE ..."
security export -k "$KEYCHAIN" -t identities -f pkcs12 -P "$PASSWORD" -o "$OUTPUT_FILE"

echo "=================================================="
echo "✅ 证书成功导出: $OUTPUT_FILE"
echo "🔑 导出密码: $PASSWORD"
echo "=================================================="
echo ""
echo "👉 【如何在其他 Mac 或 CI/CD 打包机上导入使用】"
echo "1. 将该 $OUTPUT_FILE 文件复制到打包机上，运行:"
echo "   ./scripts/import_certificate.sh \"$OUTPUT_FILE\" \"$PASSWORD\""
echo ""
echo "2. 若在 GitHub Actions / GitLab CI 中使用 (Base64 环境变量):"
echo "   Base64 字符串已生成 (可存入 CI Secrets: MACOS_CERT_P12_BASE64):"
base64 -i "$OUTPUT_FILE" | tr -d '\n'
echo ""
echo ""
