module.exports = {
  content: ['./templates/**/*.html'],
  theme: {
    extend: {
	gridTemplateRows: {
		 '7': 'repeat(7, minmax(0, 1fr))',
		 '12': 'repeat(12, minmax(0, 1fr))'
	},
	gridRow: {
        	'span-8': 'span 8 / span 8',
        	'span-10': 'span 10 / span 10',
      	}
    },
  },
  plugins: [],
}
