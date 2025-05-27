using CSV,ProgressMeter,Tables,DataFrames,OrdinaryDiffEq,SteadyStateDiffEq,Catalyst, Latexify
let

# Independent variable:
@parameters t

# Parameters:
ps = @parameters I_d = 4 γ_p = 1/2400 γ_m = 0.0067 s_A = 0.0 k_crF = 0.001848 k_crUF = 1/2400 k_crB = 0.0000483  k_crunB = 1/2400  k_J23101 = 0.019 k_J23117 = 0.0016

# Species:
sps = @species G_d(t) = 9.0 md(t) d(t) S_Ap(t) g_Ap(t) S_An(t) g_An(t) c_Ap(t) c_An(t) D_A(t) = 40.0 C_Ap(t) C_An(t) C_Ap_An(t) g_Bp(t) c_Bp(t) g_Bn(t) c_Bn(t) D_B(t) = 40.0 C_Bp(t) C_Bn(t) C_Bp_Bn(t) g_Cp(t) c_Cp(t) g_Cn(t) c_Cn(t) D_C(t) = 40.0 C_Cp(t) C_Cn(t) m_C(t) Y_C(t)

# Input functions:
dtet(x)=0.0000931+(0.046-0.0000931)*((x^3.8)/((13^3.8)+(x^3.8)))
dtac(x)=0.0000912+(0.0627-0.0000912)*((x^1.8)/((140.6^1.8)+(x^1.8)))

# Reactions:

rn=@reaction_network rn begin
    @parameters I_d = 4 γ_p = 1/2400 γ_m = 0.0067 s_A = 0.0 k_crF = 0.001848 k_crUF = 1/2400 k_crB = 0.0000483  k_crunB = 1/2400  k_J23101 = 0.019 k_J23117 = 0.0016
    @species G_d(t) = 9.0 md(t) = 0.0 d(t) = 0.0 S_A(t) = 40.0 g_An(t) = 0.0 c_An(t) = 0.0 D_A(t) = 40.0 C_A_An(t) = 0.0 g_Cp(t) = 0.0 c_Cp(t) = 0.0 D_C(t) = 40.0 C_C_Cp(t) = 0.0 m_C(t) = 0.0 Y_C(t) = 0.0 
    dtet(I_d), G_d --> md + G_d
    3.59, md --> d + md
    γ_p, d --> ∅
    γ_m, md --> ∅
    dtac(s_A), S_A --> g_An + S_A
    γ_m, g_An --> ∅
    # k_crF, d + g_Ap --> c_Ap
    # k_crUF, c_Ap --> d + g_Ap
    k_crF, d + g_An --> c_An
    k_crUF, c_An --> d + g_An
    # k_crB, c_Ap + D_A --> C_A_Ap
    # k_crunB, C_A_Ap --> c_Ap + D_A
    k_crB, c_An + D_A --> C_A_An
    k_crunB, C_A_An --> c_An + D_A
    # k_crB, c_An + C_A_Ap --> C_A_Ap_An
    # k_crunB, C_A_Ap_An --> c_An + C_A_Ap
    # k_crB, c_Ap + C_A_An --> C_A_Ap_An
    # k_crunB, C_A_Ap_An --> c_Ap + C_A_An
    # γ_p, c_Ap --> ∅
    γ_p, c_An --> ∅
    k_J23101, D_A --> g_Cp + D_A
    γ_m, g_Cp --> ∅
    k_crF, d + g_Cp --> c_Cp
    k_crUF, c_Cp --> d + g_Cp
    # k_crF, d + g_Cn --> c_Cn
    # k_crUF, c_Cn --> d + g_Cn
    k_crB, c_Cp + D_C --> C_C_Cp
    k_crunB, C_C_Cp --> c_Cp + D_C
    # k_crB, c_Cn + D_C --> C_C_Cn
    # k_crunB, C_C_Cn --> c_Cn + D_C
    # k_crB, c_Cn + C_C_Cp --> C_C_Cp_Cn
    # k_crunB, C_C_Cp_Cn --> c_Cn + C_C_Cp
    # k_crB, c_Cp + C_C_Cn --> C_C_Cp_Cn
    # k_crunB, C_C_Cp_Cn --> c_Cp + C_C_Cn
    γ_p, c_Cp --> ∅
    # γ_p, c_Cn --> ∅
    k_J23117, D_C --> m_C + D_C
    22k_J23117, C_C_Cp --> m_C + C_C_Cp
    # (22//9)*k_J23117, C_C_Cp_Cn --> m_C + C_C_Cp_Cn
    # (1//9)*k_J23117, C_C_Cn --> m_C + C_C_Cn
    1.22, m_C --> Y_C + m_C
    γ_m, m_C --> ∅
    γ_p, Y_C --> ∅
end

# Produces latex version of equations:

print(latexify(rn))

# Uncomment below if you would like to run the model:

# Define your output dictionary:

out_dict="."

# Function to convert model states to a dictionary:

std(i)=try return Dict(i=>0.0 for i in states(i)) catch NA end

# Defines the induction:

p1=[:s_A => 1000]
p2=[:s_A => 0]

# Intial parameters: 

tspan=(0,216000)
u02=std(rn)

# Convert the reaction network to an ODE problem for solving:

oprob = ODEProblem(rn, [],tspan,p1)

# Solve to an intial steady state without the induction:

prob = SteadyStateProblem(oprob)
u02 = solve(prob, DynamicSS(Rodas4P(),tspan=Inf))

# Final solve and write to file: 

oprob2 = remake(oprob;u0=u02,p=p2)
CSV.write("$(out_dict)/data.gz", DataFrame(solve(oprob2,Rodas4P(),saveat=60)), compress = true)

end