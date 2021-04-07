#[macro_use] extern crate mime;

use iron::prelude::*;
use iron::status;
use router::Router;
use urlencoded::UrlEncodedBody;

use bio::io::fasta::Reader;

use regex::Regex;
use lazy_static::lazy_static;

use open;

use std::str;
use std::io;

enum DirectionKind
{
	Uniprot2Phytozome,
	Phytozome2Uniprot,
}

struct Conversion
{
	from: String,
	to: String,
	sequence: Vec<u8>,
}

impl Conversion
{
	fn from(from: String, to: String, sequence: Vec<u8>) -> Conversion
	{
		Conversion
		{
			from,
			to,
			sequence,
		}
	}

	fn find_matches(ids: Vec<&str>, direction: DirectionKind) -> Result<Vec<Conversion>, io::Error>
	{
		let from_db;
		let to_db;

		match direction
		{
			DirectionKind::Uniprot2Phytozome =>
			{
				from_db = Reader::from_file("uniprot_db.fasta");
				to_db = Reader::from_file("phytozome_db.fasta");
			}
			DirectionKind::Phytozome2Uniprot =>
			{
				from_db = Reader::from_file("phytozome_db.fasta");
				to_db = Reader::from_file("uniprot_db.fasta");
			}
		}

		let from_db = match from_db
		{
	        Ok(db) => db,
	        Err(error) => return Err(io::Error::new(io::ErrorKind::NotFound, error.to_string())),
	    };

		let to_db = match to_db
		{
	        Ok(db) => db,
	        Err(error) => return Err(io::Error::new(io::ErrorKind::NotFound, error.to_string())),
	    };

		let mut results: Vec<Conversion> = Vec::new();

		for from_record in from_db.records()
		{
			let from_record = from_record.unwrap();

			for id in &ids
			{
				let in_id = from_record.id().contains(id);
				let in_desc = from_record.desc().unwrap().contains(id);
				if in_id || in_desc
				{
					let mut sequence = from_record.seq().to_vec();

					match direction
					{
						DirectionKind::Uniprot2Phytozome =>
						{
							sequence.push(42); // Add an asterisk to match the phytozome sequence
						}
						DirectionKind::Phytozome2Uniprot =>
						{
							sequence.pop();// Remove an asterisk to match the uniprot sequence
						}
					}

					sequence.make_ascii_uppercase();
					results.push(Conversion::from(id.to_string(), String::from("Unknown"), sequence));
					break;
				}
			}
		}

		for to_record in to_db.records()
		{
			let to_record = to_record.unwrap();
			for record in results.iter_mut()
			{
				if to_record.seq() == record.sequence
				{
					lazy_static!
					{
						static ref TRANSCRIPT_REGEX: Regex = Regex::new(r"transcript=(?P<transcript>[a-zA-Z][0-9a-zA-Z_.]*)").unwrap();
					}

					match direction
					{
						DirectionKind::Uniprot2Phytozome =>
						{
							let captures = TRANSCRIPT_REGEX.captures(to_record.desc().unwrap()).unwrap();
							record.sequence.pop(); // Remove the asterisk to make output prettier

							// TODO: use less string conversions
							record.to = String::from(captures.name("transcript").unwrap().as_str());
						}
						DirectionKind::Phytozome2Uniprot =>
						{
							let split: Vec<&str> = to_record.id().split("|").collect();

							// TODO: use less string conversions
							record.to = String::from(split[1]);
						}
					}
					break;
				}
			}
		}

		Ok(results)
	}
}

fn post_u2p(_req: &mut Request) -> IronResult<Response>
{
	let mut response = Response::new();
	let direction_string: &str;

	let form_data = match _req.get_ref::<UrlEncodedBody>()
	{
		Err(e) =>
		{
			response.set_mut(status::BadRequest);
			response.set_mut(format!("Error parsing form data: {:?}\n", e));
			return Ok(response);
		}

		Ok(map) => map
	};

	let unparsed_ids = match form_data.get("id_value")
	{
		None =>
		{
			response.set_mut(status::BadRequest);
			response.set_mut(format!("Form data has no 'id_value' parameter\n"));
			return Ok(response);
		}

		Some(nums) => nums
	};

	let direction = match form_data.get("direction")
	{
		None =>
		{
			response.set_mut(status::BadRequest);
			response.set_mut(format!("Form data has no 'direction' parameter\n"));
			return Ok(response);
		}

		Some(value) if value[0] == "u2p" =>
		{
			direction_string = " [UNIPROT=";
			DirectionKind::Uniprot2Phytozome
		}

		Some(value) if value[0] == "p2u" =>
		{
			direction_string = " [PHYTOZOME=";
			DirectionKind::Phytozome2Uniprot
		}

		Some(_) =>
		{
			direction_string = " [UNIPROT=";
			DirectionKind::Uniprot2Phytozome
		}
	};

	let display_sequences = match form_data.get("sequences")
	{
		None => false,
		Some(_value) => true,
	};

	let mut id_values = Vec::new();
	for unparsed in unparsed_ids
	{
		lazy_static!
		{
			static ref SPLIT_REGEX: Regex = Regex::new(r"[ \t,\r\n]+").unwrap();
		}

		let mut split_values = SPLIT_REGEX.split(unparsed.trim()).collect();
		id_values.append(&mut split_values);
	}

	id_values.retain(|&element| element != ""); // remove empty values

	let mut final_result = String::new();
	let mut final_html_result = String::new();

	let converted_results = Conversion::find_matches(id_values, direction);
	let converted_results = match converted_results
	{
        Ok(results) => results,
        Err(error) =>
		{
			final_html_result = error.to_string();

			Vec::new()
		},
    };

	if converted_results.is_empty()
	{
		final_html_result.push_str("No results found.");
	}

	for result in converted_results
	{
		final_result.push_str(">");
		final_result.push_str(&result.to);
		final_result.push_str(direction_string);
		final_result.push_str(&result.from);
		final_result.push_str("]\n");

		final_html_result.push_str(">");
		final_html_result.push_str(&result.to);
		final_html_result.push_str(direction_string);
		final_html_result.push_str(&result.from);
		final_html_result.push_str("]<br/>");

		let protein_sequence = textwrap::fill(str::from_utf8(&result.sequence).unwrap(), 60);
		final_result.push_str(protein_sequence.as_str());
		final_result.push_str("\n");
		if display_sequences == true
		{
			final_html_result.push_str(protein_sequence.replace("\n", "<br>").as_str());
			final_html_result.push_str("<br/>");
		}

		final_result.push_str("\n");
		final_html_result.push_str("<br/>");
	}

	let final_html_result = format!("<input type='hidden' id='fasta_output' name='fasta_output' value='{}' readonly><p style='font-family: Courier New;'>{}</p>", final_result, final_html_result);

	response.set_mut(status::Ok);
	response.set_mut(mime!(Text/Html; Charset=Utf8));
	response.set_mut(build_html(format!("
		Results:<br/>
		{}
		{}
		{}
		", final_html_result, r#"<input type="button" id="bt" value="Download FASTA" onclick="saveFile()" />"#, r#"<button onclick="document.location='/'">Convert More</button>"#)));

	Ok(response)
}

fn get_form(_req: &mut Request) -> IronResult<Response>
{
	let mut response = Response::new();

	response.set_mut(status::Ok);
	response.set_mut(mime!(Text/Html; Charset=Utf8));
	response.set_mut(build_html(r#"
		<form action="/u2p" method="post">
			A simple web based tool to convert Uniprot IDs into Phytozome IDs and vice versa.<br/>
			© Eilidh Ward 2021<br/><br/>

			<strong>DISCLAIMER: No warranty is provided with this tool.<br/>
			I will not be held responsible for any damage or loss of work.<br/>
			Use at your own risk.</strong><br/><br/>
			Expectations:<br/>
			<ul>
  				<li>A downloaded Uniprot proteome of species X in FASTA format named 'uniprot_db.fasta'</li>
  				<li>A downloaded Phytozome proteome of species X in FASTA format named 'phytozome_db.fasta'</li>
  				<li>A list of identifiers you wish to convert, seperated by spaces, new lines or commas in the text field below</li>
			</ul>
			Identifiers:<br/>
			<textarea rows="10" cols="50" name="id_value" id="txt"></textarea>
			<br/><br/>
			Direction:<br/>
			<input type="radio" id="u2p" name="direction" value="u2p" checked>
			<label for="u2p">Uniprot to Phytozome</label><br>
			<input type="radio" id="p2u" name="direction" value="p2u">
			<label for="p2u">Phytozome to Uniprot</label><br><br/>

			<input type="checkbox" id="display_sequences" name="sequences" value="Sequences">
			<label for="display_sequences">Display Sequences?</label><br><br/>
			<button type="submit" id="btSubmit">Convert</button>
		</form>
		"#.to_string()));

	Ok(response)
}

// TODO: Change to a macro
fn build_html(text: String) -> String
{
	static RAW_STRING_HEADER: &str= r#"
		<!DOCTYPE html>
		<html>
		<head>
			<title>Uniphy: Uniprot <-> Phytozome Converter</title>
			<style>
			body { font-family: Arial; }
			</style>
			<script>
				let action = () => {
					text_value = document.getElementById('txt').value;
					var bt = document.getElementById('btSubmit');
			        if (text_value != '')
					{
			            bt.disabled = false;
			        }
			        else
					{
			            bt.disabled = true;
			        }
			    }

			    let saveFile = () => {

			        // Get the data from each element on the form.
			    	let data = document.getElementById('fasta_output').value;

			        // Convert the text to BLOB.
			        const textToBLOB = new Blob([data], { type: 'text/plain' });
			        const sFileName = 'results.fasta';	   // The file to save the data.

			        let newLink = document.createElement("a");
			        newLink.download = sFileName;

			        if (window.webkitURL != null) {
			            newLink.href = window.webkitURL.createObjectURL(textToBLOB);
			        }
			        else {
			            newLink.href = window.URL.createObjectURL(textToBLOB);
			            newLink.style.display = "none";
			            document.body.appendChild(newLink);
			        }

			        newLink.click();
			    }
			</script>
		</head>
		<body>
		<h2>Uniphy</h2>
		<h3>A <a href="https://www.uniprot.org/" target="_blank">Uniprot</a> ID <-> <a href="https://phytozome.jgi.doe.gov/pz/portal.html" target="_blank">Phytozome</a> Transcript ID Converter</h3>"#;

	static RAW_STRING_FOOTER: &str= r#"
		</body>
		</html>
		"#;

	format!("{}{}{}", RAW_STRING_HEADER, text, RAW_STRING_FOOTER)
}

fn main()
{
	let mut router = Router::new();
	router.get("/", get_form, "root");
	router.post("/u2p", post_u2p, "u2p");

    println!("Starting server on http://localhost:3000");
    println!("");

	match open::that("http://localhost:3000")
	{
	    Ok(exit_status) =>
		{
	        if exit_status.success()
			{
	            println!("You may need to refresh your browser if it");
	            println!("does not display correctly on first start.");
	        }
			else
			{
	            if let Some(code) = exit_status.code()
				{
	                println!("Command returned non-zero exit status {}!", code);
	            }
				else
				{
	                println!("Command returned with unknown exit status!");
	            }
	        }
	    }

	    Err(why) => println!("Failure to execute command: {}", why),
	}

	Iron::new(router).http("localhost:3000").unwrap();
}
