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
    echo "❌ 错误: 未在钥匙串中找到名为 \"$CERT_NAME\" 的有效证书身份。"
    echo "请先运行 ./scripts/setup_certificate.sh 生成本地证书。"
    exit 1
fi

echo "正在从钥匙串精准提取 \"$CERT_NAME\" 并导出至: $OUTPUT_FILE ..."

# 使用 Python 与 OpenSSL 进行确定性唯一证书及匹配私钥提取，杜绝导出其他无关证书
python3 - "$CERT_NAME" "$OUTPUT_FILE" "$PASSWORD" "$KEYCHAIN" << 'EOF'
import sys, os, subprocess, tempfile, re

cert_name = sys.argv[1]
output_file = sys.argv[2]
password = sys.argv[3]
keychain = sys.argv[4]

# 1. 提取目标证书 PEM
res_cert = subprocess.run(
    ["security", "find-certificate", "-c", cert_name, "-p", keychain],
    capture_output=True, text=True
)
if res_cert.returncode != 0 or not res_cert.stdout.strip():
    print(f"❌ 无法从钥匙串获取证书 PEM: {cert_name}", file=sys.stderr)
    sys.exit(1)

target_cert_pem = res_cert.stdout

# 计算目标证书的公钥模数 (Modulus)
res_mod = subprocess.run(
    ["openssl", "x509", "-noout", "-modulus"],
    input=target_cert_pem, capture_output=True, text=True, check=True
)
target_modulus = res_mod.stdout.strip()

# 2. 导出所有身份至临时私有文件，提取匹配私钥
temp_pass = "temp-export-pass-" + os.urandom(8).hex()
with tempfile.NamedTemporaryFile(suffix=".p12", delete=False) as tf:
    temp_p12 = tf.name

try:
    subprocess.run(
        ["security", "export", "-k", keychain, "-t", "identities", "-f", "pkcs12", "-P", temp_pass, "-o", temp_p12],
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=True
    )

    # 提取临时 p12 中的全部私钥
    p = subprocess.run(
        ["openssl", "pkcs12", "-in", temp_p12, "-nocerts", "-nodes", "-passin", f"pass:{temp_pass}", "-legacy"],
        capture_output=True, text=True
    )
    if p.returncode != 0:
        # 兼容无 -legacy 标志的 OpenSSL 版本
        p = subprocess.run(
            ["openssl", "pkcs12", "-in", temp_p12, "-nocerts", "-nodes", "-passin", f"pass:{temp_pass}"],
            capture_output=True, text=True, check=True
        )

    keys = re.findall(r"(-----BEGIN (?:RSA )?PRIVATE KEY-----[\s\S]+?-----END (?:RSA )?PRIVATE KEY-----)", p.stdout)
    matching_key = None
    for k in keys:
        kres = subprocess.run(["openssl", "rsa", "-noout", "-modulus"], input=k, capture_output=True, text=True)
        if kres.returncode == 0 and kres.stdout.strip() == target_modulus:
            matching_key = k
            break

    if not matching_key:
        print(f"❌ 致命错误: 未在导出的身份中匹配到证书 \"{cert_name}\" 的配对私钥！", file=sys.stderr)
        sys.exit(1)

    # 3. 将唯一定确的证书与私钥重新打包为标准 .p12 格式
    with tempfile.NamedTemporaryFile("w+", delete=False) as fc, tempfile.NamedTemporaryFile("w+", delete=False) as fk:
        fc.write(target_cert_pem)
        fc.flush()
        fk.write(matching_key)
        fk.flush()
        cert_path = fc.name
        key_path = fk.name

    try:
        # 导出确定性 .p12
        cmd = [
            "openssl", "pkcs12", "-export",
            "-inkey", key_path,
            "-in", cert_path,
            "-out", output_file,
            "-name", cert_name,
            "-passout", f"pass:{password}",
            "-legacy"
        ]
        ret = subprocess.run(cmd, capture_output=True, text=True)
        if ret.returncode != 0:
            # 兼容非 legacy 格式
            cmd.remove("-legacy")
            subprocess.run(cmd, check=True)
    finally:
        if os.path.exists(cert_path): os.remove(cert_path)
        if os.path.exists(key_path): os.remove(key_path)

finally:
    if os.path.exists(temp_p12):
        os.remove(temp_p12)
EOF

echo "=================================================="
echo "✅ 证书成功导出 (确定性单身份): $OUTPUT_FILE"
echo "📏 文件大小: $(ls -lh "$OUTPUT_FILE" | awk '{print $5}')"
echo "🔑 导出密码: $PASSWORD"
echo "=================================================="
echo ""
echo "👉 【如何在 GitHub Actions / CI 中使用】"
echo "1. 复制 Base64 字符串至剪贴板 (已自动执行 pbcopy):"
base64 -i "$OUTPUT_FILE" | tr -d '\n' | pbcopy 2>/dev/null || true
echo "   (已通过 pbcopy 复制到剪贴板，若未复制可直接使用下方内容)"
echo ""
base64 -i "$OUTPUT_FILE" | tr -d '\n'
echo ""
echo ""
echo "2. 前往 GitHub 仓库 -> Settings -> Secrets and variables -> Actions"
echo "   更新 Secret: MACOS_CERT_P12_BASE64"
echo ""
