import os
import json
import csv
import io
import zipfile
import requests
from google import genai
from google.genai import types

client = genai.Client()

def generate_anchors_with_gemini(query: str, num: int) -> tuple[str, list[str]]:
  prompt = f"""
  I want you to take this query, "{query}", and create sentences that would be an example of sentences that the query is ultimately looking for. For example, if the query is "Find all instances of X talking about Y", I want you take create sentences like "You guys ever hear about Y and how useful it is?" or "I can't believe you guys aren't on Y", etc... The output you create should be exactly like this:

  Generate a JSON object with exactly two keys:
    1. "category": A short, snake_case string summarizing the topic (max 3 words), the categories for each set should be distinct from each other. Try to make the categories specific subject, person or thing in the set of sentences versus a more general concept. The more specific the category title, the better.
    2. "variations": A list of exactly {num} distinct sentences that would "match" the query, like the examples I gave you before. We need {num} examples.
    """

  try:
    response = client.models.generate_content(
      model='gemini-2.5-flash',
      contents=prompt,
      config=types.GenerateContentConfig(
        response_mime_type="application/json",
        temperature=0.7,
      )
    )

    result = json.loads(response.text)
    category = result.get("category", "general_query")
    variations = result.get("variations", [query] * num)
  except Exception as e:
    print(f"Gemini API Error: {e}")
    fallback_category = query.replace(" ", "_").lower()[:20]
    return fallback_category, [query] * num

  return category, variations

def generate_anchors_from_fields(fields: dict, num: int) -> tuple[str, list[str]]:
  prompt = f"""
  I have a structured search request with the following criteria:
    - Outcome: {fields.get('desiredOutcome', 'N/A')}
    - Date: {fields.get('importantDate', 'N/A')}
    - People: {fields.get('importantPeople', 'N/A')}
    - Events: {fields.get('importantEvents', 'N/A')}
    - Location: {fields.get('importantLocation', 'N/A')}

    Create a series of {num * num} sentences that would be examples of sentences that match the overall criteria of these fields as queries. There might be more than one value for each specific criteria (like multiple "People" or multiple "Location". For each extra value can you please generate an extra set of {num} distinct sentences. In terms of specific expected, a great example could be "I remember I saw 'People[0]' the other day at 'Events[0]' and it was awesome, the whole place went 'Outcome[0]'. If I remember correctly, it was at the 'Location[0]'". Distinct example sentences along those lines would be perfect.

    Generate a JSON object with exactly two keys:
      1. "category": A short, snake_case string summarizing the topic of the set of sentences (max 3 words), the categories for each set should be distinct from each other. Try to make the categories specific subject, person or thing in the set of sentences versus a more general concept. The more specific the category title, the better.
      2. "variations": A list of roughly {num * num}, except when extra values exist, distinct sentences that would "match" this criteria.
      """
  try:
    response = client.models.generate_content(
      model='gemini-2.5-flash',
      contents=prompt,
      config=types.GenerateContentConfig(response_mime_type="application/json", temperature=0.7)
    )
    result = json.loads(response.text)
    return result.get("category", "structured_query"), result.get("variations", [])
  except Exception as e:
    print(f"Gemini API Error: {e}")
    return "structured_query", ["Error processing fields"] * num

def break_nl_into_queries(nl_string: str) -> list[str]:
  prompt = f"""
  Take the following natural language search description and break it down into a list of distinct, logical search queries.
  Description: "{nl_string}"

  Return a JSON object with a single key "queries" containing a list of strings representing the distinct sub-queries.
  """
  try:
    response = client.models.generate_content(
      model='gemini-2.5-flash',
      contents=prompt,
      config=types.GenerateContentConfig(response_mime_type="application/json", temperature=0.4)
    )
    result = json.loads(response.text)
    return result.get("queries", [nl_string])
  except Exception as e:
    print(f"Gemini API Error breaking down NL: {e}")
    return [nl_string]

def process_queries(queries: dict, data_csv_path: str, target_col_index: int, endpoint_url: str):
  keywords_buffer = io.StringIO()
  writer = csv.writer(keywords_buffer)

  use_nl = queries.get("usedNaturalLanguage", False)

  print("Sending queries to Gemini and building payload...")

  if use_nl:
    nl_text = queries.get("naturalLanguageDescription", "")
    queries = break_nl_into_queries(nl_text)
    print(f"Broke NL down into {len(queries)} distinct queries.")

    for query in queries:
      category, variations = generate_anchors_with_gemini(query, num=5)
      writer.writerow([category] + variations)
      print(f"Generated anchors for NL query: {category}")

  else:
    fields = queries.get("fields", {})
    category, variations = generate_anchors_from_fields(fields, num=5)
    writer.writerow([category] + variations)
    print(f"Generated anchors for Fields: {category}")

  keywords_csv_data = keywords_buffer.getvalue().encode('utf-8')

  print(f"\nPackaging {data_csv_path} and keywords into payload...")
  zip_buffer = io.BytesIO()
  with zipfile.ZipFile(zip_buffer, 'w', zipfile.ZIP_DEFLATED) as zip_file:
    zip_file.writestr("UPLOAD/keywords.csv", keywords_csv_data)
    zip_file.write(data_csv_path, arcname="UPLOAD/input_data.csv")

  zip_buffer.seek(0)
  filename = f"upload_{target_col_index}.zip"
  files = {'file': (filename, zip_buffer, 'application/zip')}

  print(f"\nSending queries to {endpoint_url}...")
  try:
    response = requests.post(endpoint_url, files=files, timeout=3600)

    if response.status_code == 200:
      output_filename = "DOWNLOAD/filtered_results.zip"
      with open(output_filename, "wb") as f:
        f.write(response.content)
      print(f"Success! Saved results locally as {output_filename}")

      return output_filename
    else:
      print(f"Error {response.status_code}: {response.text}")
  except requests.exceptions.ConnectionError:
    print("Connection Error: Could not reach the Rust server. Is it running?")

