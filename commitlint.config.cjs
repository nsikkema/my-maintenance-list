module.exports = {
    extends: ['@commitlint/config-conventional'],
    defaultIgnores: false,
    rules: {
        'body-max-line-length': [0, 'always', '120']
    }
};
