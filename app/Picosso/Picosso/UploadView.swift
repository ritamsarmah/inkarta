//
//  UploadView.swift
//  Picosso
//
//  Created by Ritam Sarmah on 3/23/23.
//

import SwiftUI

struct UploadView: View {
    
    @Environment(\.dismiss) var dismiss
    
    @ObservedObject var viewModel: UploadViewModel
    
    var body: some View {
        NavigationView {
            VStack {
                switch viewModel.imageState {
                case .success(let image):
                    image.resizable()
                        .aspectRatio(contentMode: .fit)
                        .frame(maxHeight: 400)
                case .loading:
                    ProgressView()
                case .empty:
                    Image(systemName: "person.fill")
                        .font(.system(size: 40))
                        .foregroundColor(.white)
                case .failure:
                    Image(systemName: "exclamationmark.triangle.fill")
                        .font(.system(size: 40))
                        .foregroundColor(.white)
                }
                Form {
                    Section {
                        TextField("Title", text: $viewModel.title)
                        TextField("Artist", text: $viewModel.artist)
                        Toggle("Use Padding", isOn: $viewModel.shouldPad)
                    }
                    Section {
                        Toggle("Overwrite Existing Art", isOn: $viewModel.shouldOverwrite)
                    }
                }
            }
            .toolbar {
                ToolbarItem(placement: .navigationBarLeading) {
                    Button("Cancel", role: .cancel) {
                        dismiss()
                    }
                }
                
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Upload") {
                        Task {
                            let success = await viewModel.upload()
                            if success {
                                dismiss()
                            }
                        }
                    }
                    .disabled(viewModel.title.isEmpty || viewModel.artist.isEmpty)
                }
            }
        }
        .alert("Error", isPresented: $viewModel.isShowingErrorAlert, actions: {
            Button("OK", role: .cancel) {
                viewModel.errorMessage = nil
            }
        }, message: {
            Text(viewModel.errorMessage ?? "")
        })
    }
}

//struct UploadView_Previews: PreviewProvider {
//    static var previews: some View {
//        UploadView()
//    }
//}
