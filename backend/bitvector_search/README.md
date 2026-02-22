This program is called Bitvector Search, and is the part of the backend that takes 
in the user queries plus user data packaged as a single zipfile and returns a single 
zipfile containing the smaller, categorized dataset as a csv. The program is hosted on 
Google Cloud Run and is is accessible by calling (endpoint/process).

There is an expected format, the program expects a single zipfile as the input data. The zipfile
will contain two csv files: a user data file and a user "keywords" file that contains the queries.
The user data filename must end on the index of the column of data that will be processed against user queries.
For example, if my user data csv file is formatted as "Subject\_line, timestamp, message\_body", and 
the "message\_body" column is the one I want to process against user queries, the filename for the 
user data csv file must be formatted as "data\_2.zip", because "message\_body" is in column 2.

The user queries filename must contain the word "keyword" in it. It's probably more reliable to set the
query filename to "keywords.csv" as the program uses the word "keyword" to differentiate between data and
query files.

The program will return a single zipfile containing categorized dataset of only the rows from the original 
dataset that matched any of the queries. The return format will look like this:

{category | score | processed\_data}

Score is the value of how close the processed-data was found to match the user query. Category is a value
that should be defined before hand. The user queries should be pre-processed to have the following format:

{category | user\_query1 | user\_query2 | etc...}


