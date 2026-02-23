-- Initial schema for Blockchain Document Storage (MySQL/TiDB)

-- Users table
CREATE TABLE IF NOT EXISTS users (
  id INT AUTO_INCREMENT PRIMARY KEY,
  open_id VARCHAR(255) NOT NULL DEFAULT '',
  name VARCHAR(255) NULL,
  email VARCHAR(255) NULL UNIQUE,
  password_hash VARCHAR(255) NULL,
  login_method VARCHAR(50) NULL,
  role VARCHAR(50) NOT NULL DEFAULT 'user',
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  last_signed_in TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Documents table
CREATE TABLE IF NOT EXISTS documents (
  id INT AUTO_INCREMENT PRIMARY KEY,
  user_id INT NOT NULL,
  file_name VARCHAR(512) NOT NULL,
  document_hash VARCHAR(64) NOT NULL,
  file_size BIGINT NOT NULL,
  mime_type VARCHAR(255) NOT NULL,
  s3_url TEXT NOT NULL,
  status VARCHAR(50) NOT NULL DEFAULT 'pending',
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  INDEX idx_documents_user_id (user_id),
  UNIQUE KEY uq_documents_hash (document_hash),
  CONSTRAINT fk_documents_user FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Blockchain records table (camelCase name in README; snake_case in DB is recommended)
CREATE TABLE IF NOT EXISTS blockchain_records (
  id INT AUTO_INCREMENT PRIMARY KEY,
  document_id INT NOT NULL,
  transaction_hash VARCHAR(66) NOT NULL,
  block_number BIGINT NOT NULL DEFAULT 0,
  status VARCHAR(50) NOT NULL DEFAULT 'pending',
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  INDEX idx_blockchain_records_document_id (document_id),
  CONSTRAINT fk_blockchain_records_document FOREIGN KEY (document_id) REFERENCES documents(id)
);

-- Document transfer history
CREATE TABLE IF NOT EXISTS document_transfers (
  id INT AUTO_INCREMENT PRIMARY KEY,
  document_id INT NOT NULL,
  from_user_id INT NOT NULL,
  to_user_id INT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  INDEX idx_document_transfers_document_id (document_id),
  CONSTRAINT fk_document_transfers_document FOREIGN KEY (document_id) REFERENCES documents(id),
  CONSTRAINT fk_document_transfers_from_user FOREIGN KEY (from_user_id) REFERENCES users(id),
  CONSTRAINT fk_document_transfers_to_user FOREIGN KEY (to_user_id) REFERENCES users(id)
);

