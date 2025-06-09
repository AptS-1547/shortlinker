import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

// 获取 __dirname 的 ES 模块等价物
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function getGitVersion() {
  try {
    // 尝试获取最新的 git tag
    const tag = execSync('git describe --tags --abbrev=0', { encoding: 'utf8' }).trim();

    // 获取当前提交的短哈希
    const commitHash = execSync('git rev-parse --short HEAD', { encoding: 'utf8' }).trim();

    // 检查当前是否有未提交的更改
    const isDirty = execSync('git status --porcelain', { encoding: 'utf8' }).trim() !== '';

    // 检查当前提交是否就是 tag 指向的提交
    const tagCommit = execSync(`git rev-list -n 1 ${tag}`, { encoding: 'utf8' }).trim();
    const currentCommit = execSync('git rev-parse HEAD', { encoding: 'utf8' }).trim();

    let version = tag;

    // 如果当前提交不是 tag 指向的提交，添加提交信息
    if (tagCommit !== currentCommit) {
      // 获取从 tag 到当前提交的提交数量
      const commitCount = execSync(`git rev-list --count ${tag}..HEAD`, { encoding: 'utf8' }).trim();
      version = `${tag}-${commitCount}-g${commitHash}`;
    }

    // 如果有未提交的更改，添加 dirty 标记
    if (isDirty) {
      version += '-dirty';
    }

    return version;
  } catch (error) {
    console.warn('Warning: Could not get git version info:', error.message);

    // 回退方案：使用 package.json 中的版本号
    try {
      const packageJson = JSON.parse(fs.readFileSync(path.join(__dirname, '../package.json'), 'utf8'));
      return packageJson.version || '1.0.0';
    } catch (err) {
      console.warn('Warning: Could not read package.json version:', err.message);
      return '1.0.0-unknown';
    }
  }
}

function getBuildTime() {
  return new Date().toISOString();
}

function getGitInfo() {
  try {
    const branch = execSync('git rev-parse --abbrev-ref HEAD', { encoding: 'utf8' }).trim();
    const commitHash = execSync('git rev-parse HEAD', { encoding: 'utf8' }).trim();
    const commitShort = execSync('git rev-parse --short HEAD', { encoding: 'utf8' }).trim();
    const commitDate = execSync('git log -1 --format=%ci', { encoding: 'utf8' }).trim();

    return {
      branch,
      commitHash,
      commitShort,
      commitDate
    };
  } catch (error) {
    console.warn('Warning: Could not get git info:', error.message);
    return {
      branch: 'unknown',
      commitHash: 'unknown',
      commitShort: 'unknown',
      commitDate: 'unknown'
    };
  }
}

function generateVersionInfo() {
  const version = getGitVersion();
  const buildTime = getBuildTime();
  const gitInfo = getGitInfo();

  const versionInfo = {
    version,
    buildTime,
    git: gitInfo
  };

  return versionInfo;
}

function writeVersionFile(versionInfo) {
  const versionFilePath = path.join(__dirname, '../src/version.json');

  try {
    fs.writeFileSync(versionFilePath, JSON.stringify(versionInfo, null, 2), 'utf8');
    console.log('✅ Version info generated successfully:');
    console.log(`   Version: ${versionInfo.version}`);
    console.log(`   Build Time: ${versionInfo.buildTime}`);
    console.log(`   Git Branch: ${versionInfo.git.branch}`);
    console.log(`   Git Commit: ${versionInfo.git.commitShort}`);
  } catch (error) {
    console.error('❌ Failed to write version file:', error.message);
    process.exit(1);
  }
}

function checkVersionInfo() {
  const versionFilePath = path.join(__dirname, '../src/version.json');

  try {
    if (fs.existsSync(versionFilePath)) {
      const versionInfo = JSON.parse(fs.readFileSync(versionFilePath, 'utf8'));
      console.log('📋 Current version info:');
      console.log(JSON.stringify(versionInfo, null, 2));
    } else {
      console.log('❌ Version file not found. Run "npm run version" to generate it.');
    }
  } catch (error) {
    console.error('❌ Failed to read version file:', error.message);
  }
}

// 主逻辑 - ES 模块版本
const args = process.argv.slice(2);

if (args.includes('--check')) {
  checkVersionInfo();
} else {
  const versionInfo = generateVersionInfo();
  writeVersionFile(versionInfo);
}

// ES 模块导出
export { generateVersionInfo };
