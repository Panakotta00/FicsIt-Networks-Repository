/** @type {import('tailwindcss').Config} */
module.exports = {
    content: ['./templates/**/*.html'],
    darkMode: "selector",
    theme: {
        extend: {
            fontFamily: {
                flow: ['"Flow Rounded"', "sans-serif"]
            },
            colors: {
                "primary-bg": "var(--color-primary-bg)",
                "secondary-bg": "var(--color-secondary-bg)",
                "primary-fg": "var(--color-primary-fg)",
                "secondary-fg": "var(--color-secondary-fg)",
                "accent1": "var(--color-accent1)",
                "accent2": "var(--color-accent2)",
            }
        },
    },
};
