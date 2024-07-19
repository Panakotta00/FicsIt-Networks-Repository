/** @type {import('tailwindcss').Config} */
module.exports = {
    content: ['./templates/**/*.html'],
    theme: {
        extend: {
            fontFamily: {
                flow: ['"Flow Rounded"', "sans-serif"]
            }
        },
    },
};
