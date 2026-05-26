package server

import (
	"fmt"
	"os"
	"path/filepath"
)

func CreateMicroBranch(sessionID, targetDir string) (string, error) {
	if _, err := os.Stat(targetDir); os.IsNotExist(err) {
		return "", fmt.Errorf("BranchMgr: Target directory %s does not exist", targetDir)
	}

	branchDir := filepath.Join("/tmp/sass_branches", sessionID)
	if err := os.MkdirAll(branchDir, 0755); err != nil {
		return "", fmt.Errorf("BranchMgr: Failed to create branch dir: %v", err)
	}

	if err := buildSymlinkTree(targetDir, branchDir); err != nil {
		return "", fmt.Errorf("BranchMgr: Failed to build symlink tree: %v", err)
	}

	return branchDir, nil
}

func buildSymlinkTree(src, dst string) error {
	entries, err := os.ReadDir(src)
	if err != nil {
		return err
	}

	for _, entry := range entries {
		name := entry.Name()
		srcPath := filepath.Join(src, name)
		dstPath := filepath.Join(dst, name)

		if entry.IsDir() {
			if name == "target" || name == ".git" || name == "node_modules" {
				continue
			}
			if err := os.MkdirAll(dstPath, 0755); err != nil {
				return err
			}
			if err := buildSymlinkTree(srcPath, dstPath); err != nil {
				return err
			}
		} else {
			os.Symlink(srcPath, dstPath)
		}
	}
	return nil
}
