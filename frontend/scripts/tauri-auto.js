#!/usr/bin/env node
/**
 * Auto-detect GPU and run Tauri with appropriate features
 */

const { spawnSync } = require('child_process');
const path = require('path');
const fs = require('fs');
const os = require('os');

// Get the command (dev or build)
const command = process.argv[2];
if (!command || !['dev', 'build'].includes(command)) {
  console.error('Usage: node tauri-auto.js [dev|build]');
  process.exit(1);
}

// Detect GPU feature
let feature = '';

// Check for environment variable override first
if (process.env.TAURI_GPU_FEATURE) {
  feature = process.env.TAURI_GPU_FEATURE;
  console.log(`🔧 Using forced GPU feature from environment: ${feature}`);
} else {
  try {
    const result = execSync('node scripts/auto-detect-gpu.js', {
      encoding: 'utf8',
      stdio: ['pipe', 'pipe', 'inherit']
    });
    feature = result.trim();
  } catch (err) {
    // If detection fails, continue with no features
  }
}

console.log(''); // Empty line for spacing

// Platform-specific environment variables
const platform = os.platform();
const env = { ...process.env };

if (platform === 'linux' && feature === 'cuda') {
  console.log('🐧 Linux/CUDA detected: Setting CMAKE flags for NVIDIA GPU');
  env.CMAKE_CUDA_ARCHITECTURES = '75';
  env.CMAKE_CUDA_STANDARD = '17';
  env.CMAKE_POSITION_INDEPENDENT_CODE = 'ON';
}

// Build the Tauri command as an argument list so JSON config overrides work
// consistently without shell-escaping issues.
const tauriArgs = [command];

// Release CI can provide the updater signing key. Local DMG builds should not
// fail after bundling merely because update artifacts cannot be signed.
if (command === 'build' && !env.TAURI_SIGNING_PRIVATE_KEY) {
  tauriArgs.push(
    '--config',
    JSON.stringify({ bundle: { createUpdaterArtifacts: false } })
  );
  console.log('🔓 No updater signing key found; skipping updater artifacts for this local build');
}

if (feature && feature !== 'none') {
  tauriArgs.push('--', '--features', feature);
  console.log(`🚀 Running: tauri ${command} with features: ${feature}`);
} else {
  console.log(`🚀 Running: tauri ${command} (CPU-only mode)`);
}
console.log('');

// Execute the command
try {
  const result = spawnSync('tauri', tauriArgs, { stdio: 'inherit', env });
  if (result.error) {
    throw result.error;
  }
  process.exit(result.status || 0);
} catch (err) {
  console.error(err.message || err);
  process.exit(1);
}
