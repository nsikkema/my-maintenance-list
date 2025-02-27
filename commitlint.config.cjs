module.exports = {
    extends: ['@commitlint/config-conventional'],
    defaultIgnores: false,
    rules: {
        'body-max-line-length': [2, 'always', '120']
    }
};
