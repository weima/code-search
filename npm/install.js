const fs = require('fs');
const path = require('path');
const axios = require('axios');
const tar = require('tar');
const AdmZip = require('adm-zip');
const os = require('os');

const PACKAGE_NAME = 'code-search-cli';
const BINARY_NAME = 'cs';
const VERSION = '0.1.0'; // TODO: Get this from package.json dynamically in the future
const REPO_URL = 'https://github.com/weima/code-search/releases/download';

function getPlatform() {
    const platform = os.platform();
    const arch = os.arch();

    if (platform === 'darwin') {
        return 'x86_64-apple-darwin'; // We only build x86_64 for mac currently (rosetta handles arm64)
    } else if (platform === 'linux') {
        return 'x86_64-unknown-linux-gnu';
    } else if (platform === 'win32') {
        return 'x86_64-pc-windows-msvc';
    } else {
        throw new Error(`Unsupported platform: ${platform}`);
    }
}

function getExtension() {
    return os.platform() === 'win32' ? 'zip' : 'tar.gz';
}

async function downloadBinary() {
    const target = getPlatform();
    const ext = getExtension();
    const fileName = `${PACKAGE_NAME}-v${VERSION}-${target}.${ext}`;
    const url = `${REPO_URL}/v${VERSION}/${fileName}`;
    const binDir = path.join(__dirname, 'bin');
    const outputPath = path.join(binDir, fileName);

    if (!fs.existsSync(binDir)) {
        fs.mkdirSync(binDir, { recursive: true });
    }

    console.log(`Downloading ${url}...`);

    try {
        const response = await axios({
            method: 'get',
            url: url,
            responseType: 'arraybuffer'
        });

        fs.writeFileSync(outputPath, response.data);
        console.log('Download complete. Extracting...');

        if (ext === 'zip') {
            const zip = new AdmZip(outputPath);
            zip.extractAllTo(binDir, true);
            // Windows binary is likely inside a folder or just the exe
            // We need to ensure 'cs.exe' is in binDir
        } else {
            await tar.x({
                file: outputPath,
                cwd: binDir
            });
        }

        // Cleanup archive
        fs.unlinkSync(outputPath);

        // Make executable (on unix)
        if (os.platform() !== 'win32') {
            const binPath = path.join(binDir, BINARY_NAME);
            if (fs.existsSync(binPath)) {
                fs.chmodSync(binPath, 0o755);
            }
        }

        console.log('Installation successful!');

    } catch (error) {
        console.error('Failed to download or extract binary:', error.message);
        process.exit(1);
    }
}

downloadBinary();
