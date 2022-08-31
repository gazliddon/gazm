module.exports = {
    "env": {
        "node": true,
        "es2021": true
    },
    "extends": "eslint:recommended",

    "parserOptions": {
        "ecmaVersion": "latest",
        "sourceType": "module"
    },

     "rules": {
        "no-unused-vars": ["off", { "vars": "all", "args": "after-used" }]
    },


    "globals" : {
        "alias" : true,
        "token" : true,
        "grammar" : true,
        "module" : true,
        "repeat" : true,
        "repeat1" : true,
        "choice" : true,
        "prec" : true,
        "seq" : true,
        "field" : true,
        "optional" : true
    }
}
