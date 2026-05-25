use clap::{Args, Subcommand};

use crate::soap::SoapClient;

#[derive(Args)]
pub struct StaffingArgs {
    #[command(subcommand)]
    pub command: StaffingCommand,
}

#[derive(Subcommand)]
pub enum StaffingCommand {
    /// Create a new position under a supervisory organization
    CreatePosition {
        /// WID of the supervisory organization
        #[arg(value_name = "SUPERVISORY_ORG_WID")]
        supervisory_org_wid: String,
        /// Job posting title for the new position
        #[arg(value_name = "POSITION_NAME")]
        position_name: String,
    },
    /// Hire a new employee into an existing position
    HireEmployee {
        /// Organization Reference ID of the supervisory organization
        #[arg(value_name = "SUPERVISORY_ORG_WID")]
        supervisory_org_wid: String,
        /// WID of the position to hire into
        #[arg(value_name = "POSITION_WID")]
        position_wid: String,
        /// Employee's first name
        #[arg(value_name = "FIRST_NAME")]
        first_name: String,
        /// Employee's last name
        #[arg(value_name = "LAST_NAME")]
        last_name: String,
        /// Employee's email address
        #[arg(value_name = "EMAIL")]
        email_address: String,
        /// WID of the employee type
        #[arg(value_name = "EMPLOYEE_TYPE_WID")]
        employee_type_wid: String,
        /// WID of the location
        #[arg(value_name = "LOCATION_WID")]
        location_wid: String,
        /// WID of the position time type
        #[arg(value_name = "TIME_TYPE_WID")]
        time_type_wid: String,
        /// WID of the job profile
        #[arg(value_name = "JOB_PROFILE_WID")]
        job_profile_wid: String,
    },
}

pub fn execute(args: &StaffingArgs) {
    let client = SoapClient::new().expect("Run 'init' first.");

    let result = match &args.command {
        StaffingCommand::CreatePosition {
            supervisory_org_wid,
            position_name,
        } => create_position(&client, supervisory_org_wid, position_name),
        StaffingCommand::HireEmployee {
            supervisory_org_wid,
            position_wid,
            first_name,
            last_name,
            email_address,
            employee_type_wid,
            location_wid,
            time_type_wid,
            job_profile_wid,
        } => hire_employee(
            &client,
            supervisory_org_wid,
            position_wid,
            first_name,
            last_name,
            email_address,
            employee_type_wid,
            location_wid,
            time_type_wid,
            job_profile_wid,
        ),
    };

    match result {
        Ok(body) => println!("{body}"),
        Err(e) => eprintln!("{e}"),
    }
}

fn create_position(
    client: &SoapClient,
    supervisory_org_wid: &str,
    position_name: &str,
) -> Result<String, String> {
    let request = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<env:Envelope xmlns:env="http://schemas.xmlsoap.org/soap/envelope/" xmlns:xsd="http://www.w3.org/2001/XMLSchema">
  <env:Body>
    <wd:Create_Position_Request xmlns:wd="urn:com.workday/bsvc" wd:version="v47.0">
      <wd:Business_Process_Parameters>
        <wd:Auto_Complete>true</wd:Auto_Complete>
        <wd:Run_Now>true</wd:Run_Now>
      </wd:Business_Process_Parameters>
      <wd:Create_Position_Data>
        <wd:Supervisory_Organization_Reference>
          <wd:ID wd:type="WID">{supervisory_org_wid}</wd:ID>
        </wd:Supervisory_Organization_Reference>
        <wd:Position_Data>
          <wd:Job_Posting_Title>{position_name}</wd:Job_Posting_Title>
        </wd:Position_Data>
      </wd:Create_Position_Data>
    </wd:Create_Position_Request>
  </env:Body>
</env:Envelope>"#
    );

    client.post("Staffing", "v47.0", &request)
}

#[allow(clippy::too_many_arguments)]
fn hire_employee(
    client: &SoapClient,
    supervisory_org_wid: &str,
    position_wid: &str,
    first_name: &str,
    last_name: &str,
    email_address: &str,
    employee_type_wid: &str,
    location_wid: &str,
    time_type_wid: &str,
    job_profile_wid: &str,
) -> Result<String, String> {
    let request = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<env:Envelope
    xmlns:env="http://schemas.xmlsoap.org/soap/envelope/"
    xmlns:xsd="http://www.w3.org/2001/XMLSchema">
    <env:Body>
        <wd:Hire_Employee_Request xmlns:wd="urn:com.workday/bsvc" wd:version="v47.0">
            <wd:Business_Process_Parameters>
                <wd:Auto_Complete>true</wd:Auto_Complete>
                <wd:Run_Now>true</wd:Run_Now>
            </wd:Business_Process_Parameters>
            <wd:Hire_Employee_Data>
                <wd:Organization_Reference>
                    <wd:ID wd:type="WID">{supervisory_org_wid}</wd:ID>
                </wd:Organization_Reference>
                <wd:Position_Reference>
                    <wd:ID wd:type="WID">{position_wid}</wd:ID>
                </wd:Position_Reference>
                <wd:Applicant_Data>
                    <wd:Personal_Data>
                        <wd:Name_Data>
                            <wd:Legal_Name_Data>
                                <wd:Name_Detail_Data>
                                    <wd:Country_Reference>
                                        <wd:ID wd:type="ISO_3166-1_Alpha-3_Code">USA</wd:ID>
                                    </wd:Country_Reference>
                                    <wd:First_Name>{first_name}</wd:First_Name>
                                    <wd:Last_Name>{last_name}</wd:Last_Name>
                                </wd:Name_Detail_Data>
                            </wd:Legal_Name_Data>
                        </wd:Name_Data>
                        <wd:Contact_Data>
                            <wd:Email_Address_Data>
                                <wd:Email_Address>{email_address}</wd:Email_Address>
                                <wd:Usage_Data>
                                    <wd:Type_Data wd:Primary="true">
                                        <wd:Type_Reference>
                                            <wd:ID wd:type="Communication_Usage_Type_ID">HOME</wd:ID>
                                        </wd:Type_Reference>
                                    </wd:Type_Data>
                                </wd:Usage_Data>
                            </wd:Email_Address_Data>
                        </wd:Contact_Data>
                    </wd:Personal_Data>
                </wd:Applicant_Data>
                <wd:Hire_Employee_Event_Data>
                    <wd:Employee_Type_Reference>
                        <wd:ID wd:type="WID">{employee_type_wid}</wd:ID>
                    </wd:Employee_Type_Reference>
                    <wd:Position_Details>
                        <wd:Location_Reference>
                            <wd:ID wd:type="WID">{location_wid}</wd:ID>
                        </wd:Location_Reference>
                        <wd:Position_Time_Type_Reference>
                            <wd:ID wd:type="WID">{time_type_wid}</wd:ID>
                        </wd:Position_Time_Type_Reference>
                        <wd:Job_Profile_Reference>
                            <wd:ID wd:type="WID">{job_profile_wid}</wd:ID>
                        </wd:Job_Profile_Reference>
                    </wd:Position_Details>
                </wd:Hire_Employee_Event_Data>
            </wd:Hire_Employee_Data>
        </wd:Hire_Employee_Request>
    </env:Body>
</env:Envelope>"#
    );

    client.post("Staffing", "v47.0", &request)
}
