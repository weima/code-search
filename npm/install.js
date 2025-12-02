const fs = require('fs');
const path = require('path');
const axios = require('axios');
const tar = require('tar');
const AdmZip = require('adm-zip');
const os = require('os');

const PACKAGE_NAME = 'code-search-cli';
const BINARY_NAME = 'cs';
const VERSION = '0.1.5'; // TODO: Get this from package.json dynamically in the future
const REPO_URL = 'https://github.com/weima/code-search/releases/download';

function getBinaryName() {
    const platform = os.platform();
    if (platform === 'darwin') {
        return 'cs-darwin-amd64';
    } else if (platform === 'linux') {
        return 'cs-linux-amd64';
    } else if (platform === 'win32') {
        return 'cs-windows-amd64.exe';
    } else {
        throw new Error(`Unsupported platform: ${platform}`);
    }
}

async function downloadBinary() {
    const fileName = getBinaryName();
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
        console.log('Download complete.');

        // Rename to 'cs' (or 'cs.exe')
        const finalName = os.platform() === 'win32' ? 'cs.exe' : 'cs';
        const finalPath = path.join(binDir, finalName);
        fs.renameSync(outputPath, finalPath);

        // Make executable (on unix)
        if (os.platform() !== 'win32') {
            const binPath = path.join(binDir, BINARY_NAME);
            if (fs.existsSync(binPath)) {
                fs.chmodSync(binPath, 0o755);
            }
        }

        console.log('Installation successful!');

    } catch (error) {
        console.error('Failed to download binary:', error.message);
        process.exit(1);
    }
}

downloadBinary();
