name: Manual Build and Release

on:
  workflow_dispatch:
    inputs:
      release_version:
        description: 'Release version'
        required: true
        default: '1.0.0'
      is_prerelease:
        description: 'Is this a pre-release?'
        type: boolean
        required: true
        default: false

permissions:
  contents: write

jobs:
  compile-and-publish:
    runs-on: ubuntu-latest
    steps:
    - name: Checkout code
      uses: actions/checkout@v3

    - name: Setup Rust environment
      uses: actions-rs/toolchain@v1
      with:
        profile: minimal
        toolchain: stable
        override: true

    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

    - name: Build project
      run: cargo build --release --verbose

    - name: Run tests
      run: cargo test --release --verbose

    - name: Create Release
      id: create_release
      uses: actions/create-release@v1
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      with:
        tag_name: v${{ github.event.inputs.release_version }}
        release_name: Release ${{ github.event.inputs.release_version }}
        draft: false
        prerelease: ${{ github.event.inputs.is_prerelease }}

    - name: Get artifact name
      id: get_artifact_name
      run: |
        ARTIFACT_NAME=$(ls target/release | grep -v '\.d$')
        echo "ARTIFACT_NAME=$ARTIFACT_NAME" >> $GITHUB_OUTPUT
        echo "Artifact name is: $ARTIFACT_NAME"

    - name: Upload Release Asset
      uses: actions/upload-release-asset@v1
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      with:
        upload_url: ${{ steps.create_release.outputs.upload_url }}
        asset_path: ./target/release/${{ steps.get_artifact_name.outputs.ARTIFACT_NAME }}
        asset_name: ${{ steps.get_artifact_name.outputs.ARTIFACT_NAME }}-${{ github.event.inputs.release_version }}
        asset_content_type: application/octet-stream

    - name: Generate and upload SHA256 checksum
      run: |
        cd target/release
        sha256sum ${{ steps.get_artifact_name.outputs.ARTIFACT_NAME }} > SHA256SUMS.txt
        gh release upload v${{ github.event.inputs.release_version }} SHA256SUMS.txt
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
