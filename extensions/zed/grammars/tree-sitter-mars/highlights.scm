[
  {
    "match": [
      (pair) @key
    ],
    "captures": {
      "key": {
        "highlight": "property",
        "patterns": [
          {
            "content": '"format_version"',
            "highlight": "attribute"
          },
          {
            "content": '"metadata"',
            "highlight": "attribute"
          },
          {
            "content": '"cells"',
            "highlight": "attribute"
          },
          {
            "content": '"cell_type"',
            "highlight": "attribute"
          },
          {
            "content": '"source"',
            "highlight": "attribute"
          },
          {
            "content": '"execution_count"',
            "highlight": "attribute"
          },
          {
            "content": '"language"',
            "highlight": "attribute"
          },
          {
            "content": '"title"',
            "highlight": "attribute"
          },
          {
            "content": '"dependencies"',
            "highlight": "attribute"
          },
          {
            "content": '"id"',
            "highlight": "attribute"
          }
        ]
      }
    }
  },
  {
    "match": [
      (string) @string
    ],
    "captures": {
      "string": {
        "highlight": "string"
      }
    }
  },
  {
    "match": [
      (number) @number
    ],
    "captures": {
      "number": {
        "highlight": "number"
      }
    }
  },
  {
    "match": [
      (boolean) @boolean
    ],
    "captures": {
      "boolean": {
        "highlight": "boolean"
      }
    }
  },
  {
    "match": [
      (null) @null
    ],
    "captures": {
      "null": {
        "highlight": "constant"
      }
    }
  }
]
