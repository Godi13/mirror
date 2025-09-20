import fs from 'fs/promises';
import path from 'path';
import { execSync } from 'child_process';

const rootDir = process.cwd();
const packageJsonPath = path.join(rootDir, 'package.json');
const tauriConfPath = path.join(rootDir, 'src-tauri/tauri.conf.json');
const cargoTomlPath = path.join(rootDir, 'src-tauri/Cargo.toml');
const cargoLockPath = path.join(rootDir, 'src-tauri/Cargo.lock');
const tauriDir = path.join(rootDir, 'src-tauri');

async function checkAndSyncVersions() {
  try {
    const packageJsonContent = JSON.parse(await fs.readFile(packageJsonPath, 'utf-8'));
    const masterVersion = packageJsonContent.version;

    const filesToSync = [];
    let cargoTomlWasSynced = false;

    // Check tauri.conf.json
    const tauriConfContent = JSON.parse(await fs.readFile(tauriConfPath, 'utf-8'));
    if (tauriConfContent.version !== masterVersion) {
      console.log(`[pre-commit] Version mismatch in tauri.conf.json. Syncing ${tauriConfContent.version} -> ${masterVersion}`);
      tauriConfContent.version = masterVersion;
      await fs.writeFile(tauriConfPath, JSON.stringify(tauriConfContent, null, 2) + '\n');
      filesToSync.push(tauriConfPath);
    }

    // Check Cargo.toml
    const cargoTomlContent = await fs.readFile(cargoTomlPath, 'utf-8');
    const versionRegex = /^(version\s*=\s*)"(.*?)"/m; // Corrected Regex
    const cargoVersionMatch = cargoTomlContent.match(versionRegex);
    if (!cargoVersionMatch || cargoVersionMatch[1] !== masterVersion) {
      const oldVersion = cargoVersionMatch ? cargoVersionMatch[1] : 'N/A';
      console.log(`[pre-commit] Version mismatch in Cargo.toml. Syncing ${oldVersion} -> ${masterVersion}`);
      const updatedCargoContent = cargoTomlContent.replace(versionRegex, `$1"${masterVersion}"`);
      await fs.writeFile(cargoTomlPath, updatedCargoContent);
      filesToSync.push(cargoTomlPath);
      cargoTomlWasSynced = true;
    }

    if (filesToSync.length > 0) {
      if (cargoTomlWasSynced) {
        console.log('[pre-commit] Cargo.toml changed, updating Cargo.lock...');
        try {
          execSync('cargo check', { cwd: tauriDir, stdio: 'ignore' });
          filesToSync.push(cargoLockPath);
        } catch (error) {
          console.warn('[pre-commit] Could not automatically update Cargo.lock. You may need to run "cargo check" manually in `src-tauri`.');
        }
      }

      console.log('[pre-commit] Staging synchronized version files...');
      const finalFilesToAdd = [...new Set(filesToSync)];
      execSync(`git add ${finalFilesToAdd.join(' ')}`);
      console.log('[pre-commit] Files staged. Your commit will now include the synchronized versions.');
    }

  } catch (error) {
    console.error('[pre-commit] Error during version check:', error);
    process.exit(1);
  }
}

checkAndSyncVersions();
