CREATE TABLE user_groups (
  id SERIAl PRIMARY KEY
);

ALTER TABLE dates ADD COLUMN user_group INT;
AlTER TABLE dates ADD CONSTRAINT group_foreign_key FOREIGN KEY(user_group) REFERENCES user_groups(id);
AlTER TABLE users ADD CONSTRAINT group_foreign_key FOREIGN KEY(user_group) REFERENCES user_groups(id);
