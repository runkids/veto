#!/bin/bash
# Pre-push hook to verify tag matches Cargo.toml version
# Install: cp scripts/pre-push-tag-check.sh .git/hooks/pre-push && chmod +x .git/hooks/pre-push

while read local_ref local_sha remote_ref remote_sha; do
    # Only check tag pushes
    if [[ "$local_ref" == refs/tags/v* ]]; then
        TAG_NAME=$(echo "$local_ref" | sed 's|refs/tags/||')
        TAG_VERSION=${TAG_NAME#v}
        CARGO_VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)

        echo "Checking version: tag=$TAG_VERSION, Cargo.toml=$CARGO_VERSION"

        if [ "$CARGO_VERSION" != "$TAG_VERSION" ]; then
            echo "❌ Version mismatch!"
            echo "   Tag version:       $TAG_VERSION"
            echo "   Cargo.toml version: $CARGO_VERSION"
            echo ""
            echo "Please update Cargo.toml to match the tag version:"
            echo "   version = \"$TAG_VERSION\""
            exit 1
        fi

        echo "✓ Version matched"
    fi
done

exit 0
