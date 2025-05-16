# Repository Installation Instructions

1. Add the repository GPG key:
```bash
curl -fsSL https://raw.githubusercontent.com/A5873/synx/master/repo/synx-key.asc | sudo gpg --dearmor -o /etc/apt/keyrings/synx-archive-keyring.gpg
```

2. Add the repository:
```bash
echo 'deb [signed-by=/etc/apt/keyrings/synx-archive-keyring.gpg] https://raw.githubusercontent.com/A5873/synx/master/repo noble main' | sudo tee /etc/apt/sources.list.d/synx.list
```

3. Update and install:
```bash
sudo apt update
sudo apt install synx
```
