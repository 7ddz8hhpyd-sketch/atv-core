#!/usr/bin/env bash
set -e

P12_FILE="${1:-Corvo_Development.p12}"
PASSWORD="${2:-corvo-developer}"

echo "=================================================="
echo "📥 导入通用代码签名证书 (.p12) 至当前机器钥匙串"
echo "=================================================="

if [ ! -f "$P12_FILE" ]; then
    echo "❌ 错误: 找不到证书文件: $P12_FILE"
    echo "用法: ./scripts/import_certificate.sh <证书路径.p12> [密码]"
    exit 1
fi

KEYCHAIN="$HOME/Library/Keychains/login.keychain-db"
if [ ! -f "$KEYCHAIN" ]; then
    KEYCHAIN="$HOME/Library/Keychains/login.keychain"
fi

# 如果在 CI 无头环境 (如 GitHub Actions)
if [ -n "$CI" ] || [ -n "$GITHUB_ACTIONS" ]; then
    echo "🤖 检测到 CI/CD 自动化流水线环境，创建专用构建钥匙串..."
    CI_KEYCHAIN="$RUNNER_TEMP/build.keychain"
    CI_PASSWORD="ci-temporary-password"
    security create-keychain -p "$CI_PASSWORD" "$CI_KEYCHAIN" || true
    security set-keychain-settings -lut 21600 "$CI_KEYCHAIN"
    security unlock-keychain -p "$CI_PASSWORD" "$CI_KEYCHAIN"
    security default-keychain -s "$CI_KEYCHAIN"
    KEYCHAIN="$CI_KEYCHAIN"
fi

echo "正在将 $P12_FILE 导入钥匙串: $KEYCHAIN ..."
security import "$P12_FILE" -k "$KEYCHAIN" -P "$PASSWORD" -T /usr/bin/codesign >/dev/null 2>&1

# 允许 codesign 工具免弹窗访问私钥
if [ -n "$CI" ] || [ -n "$GITHUB_ACTIONS" ]; then
    security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$CI_PASSWORD" "$KEYCHAIN" >/dev/null 2>&1 || true
    # 将 CI 钥匙串添加到全局搜索列表
    security list-keychains -d user -s "$KEYCHAIN" $(security list-keychains -d user | tr -d '"')
fi

# 提取证书并配置代码签名策略信任
TEMP_CERT=$(mktemp)
openssl pkcs12 -in "$P12_FILE" -nokeys -out "$TEMP_CERT" -passin "pass:$PASSWORD" -legacy >/dev/null 2>&1 || \
openssl pkcs12 -in "$P12_FILE" -nokeys -out "$TEMP_CERT" -passin "pass:$PASSWORD" >/dev/null 2>&1 || true

if [ -s "$TEMP_CERT" ]; then
    echo "🛡️  配置代码签名信任策略..."
    if [ -n "$CI" ] || [ -n "$GITHUB_ACTIONS" ]; then
        # CI 无头环境中，使用 sudo 将根证书加入系统钥匙串并信任，无弹窗且全局生效
        sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain "$TEMP_CERT" || true
    else
        security add-trusted-cert -r trustRoot -p codeSign -k "$KEYCHAIN" "$TEMP_CERT" >/dev/null 2>&1 || true
    fi
fi
rm -f "$TEMP_CERT"

echo "=================================================="
echo "🎉 证书导入完成！当前可用的代码签名身份:"
security find-identity -p codesigning "$KEYCHAIN"
echo "=================================================="

# 严格校验：确保钥匙串中至少存在一个可用的代码签名身份
if ! security find-identity -p codesigning "$KEYCHAIN" | grep -q "[0-9])"; then
    echo "❌ 致命错误: 证书导入失败，钥匙串中未找到任何代码签名身份！"
    exit 1
fi
